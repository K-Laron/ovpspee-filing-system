package com.ovpspeemobile

import android.Manifest
import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.ContentValues
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.provider.OpenableColumns
import com.facebook.react.bridge.ActivityEventListener
import com.facebook.react.bridge.Arguments
import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactContextBaseJavaModule
import com.facebook.react.bridge.ReactMethod
import com.facebook.react.bridge.WritableMap
import com.facebook.react.modules.core.PermissionAwareActivity
import com.facebook.react.modules.core.PermissionListener

class OvpspeeCaptureModule(private val reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext),
    ActivityEventListener {
    private var pendingPromise: Promise? = null
    private var pendingPhotoUri: Uri? = null

    init {
        reactContext.addActivityEventListener(this)
    }

    override fun getName(): String = "OvpspeeCapture"

    @ReactMethod
    fun pickFile(promise: Promise) {
        if (!beginRequest(promise)) return
        val activity = getCurrentActivity() ?: return rejectAndClear("E_NO_ACTIVITY", "No active Android activity.")
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_MIME_TYPES, ACCEPTED_MIME_TYPES)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        }

        try {
            activity.startActivityForResult(intent, PICK_FILE_REQUEST)
        } catch (error: ActivityNotFoundException) {
            rejectAndClear("E_PICKER_UNAVAILABLE", "Android file picker is unavailable.", error)
        }
    }

    @ReactMethod
    fun capturePhoto(promise: Promise) {
        if (!beginRequest(promise)) return
        if (hasRequiredCameraPermissions()) {
            launchCamera()
        } else {
            requestCameraPermissions()
        }
    }

    override fun onActivityResult(activity: Activity, requestCode: Int, resultCode: Int, data: Intent?) {
        when (requestCode) {
            PICK_FILE_REQUEST -> handleFilePickerResult(resultCode, data)
            CAPTURE_PHOTO_REQUEST -> handleCameraResult(resultCode)
        }
    }

    override fun onNewIntent(intent: Intent) = Unit

    private fun handleFilePickerResult(resultCode: Int, data: Intent?) {
        if (resultCode != Activity.RESULT_OK) {
            rejectAndClear("E_PICKER_CANCELLED", "File picker cancelled.")
            return
        }

        val uri = data?.data ?: return rejectAndClear("E_PICKER_EMPTY", "File picker did not return a file.")
        try {
            reactContext.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
        } catch (error: SecurityException) {
            rejectAndClear("E_FILE_PERMISSION", "Could not keep Android file access for upload.", error)
            return
        }

        resolveAndClear(filePayload(uri, "mobile-upload"))
    }

    private fun handleCameraResult(resultCode: Int) {
        val uri = pendingPhotoUri
        if (uri == null) {
            rejectAndClear("E_CAMERA_EMPTY", "Camera did not return a photo.")
            return
        }

        if (resultCode != Activity.RESULT_OK) {
            reactContext.contentResolver.delete(uri, null, null)
            rejectAndClear("E_CAMERA_CANCELLED", "Camera capture cancelled.")
            return
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            val values = ContentValues().apply {
                put(MediaStore.Images.Media.IS_PENDING, 0)
            }
            reactContext.contentResolver.update(uri, values, null, null)
        }

        resolveAndClear(filePayload(uri, "mobile-photo.jpg"))
    }

    private fun launchCamera() {
        val activity = getCurrentActivity() ?: return rejectAndClear("E_NO_ACTIVITY", "No active Android activity.")
        val displayName = "ovpspee-capture-${System.currentTimeMillis()}.jpg"
        val values = ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, displayName)
            put(MediaStore.Images.Media.MIME_TYPE, "image/jpeg")
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(MediaStore.Images.Media.RELATIVE_PATH, "${Environment.DIRECTORY_PICTURES}/OVPSPEE")
                put(MediaStore.Images.Media.IS_PENDING, 1)
            }
        }
        val uri = reactContext.contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values)
            ?: return rejectAndClear("E_CAMERA_STORAGE", "Could not create Android photo destination.")

        pendingPhotoUri = uri
        val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE).apply {
            putExtra(MediaStore.EXTRA_OUTPUT, uri)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
        }

        try {
            activity.startActivityForResult(intent, CAPTURE_PHOTO_REQUEST)
        } catch (error: ActivityNotFoundException) {
            reactContext.contentResolver.delete(uri, null, null)
            pendingPhotoUri = null
            rejectAndClear("E_CAMERA_UNAVAILABLE", "Android camera is unavailable.", error)
        }
    }

    private fun hasRequiredCameraPermissions(): Boolean =
        requiredCameraPermissions().all { permission ->
            Build.VERSION.SDK_INT < Build.VERSION_CODES.M ||
                reactContext.checkSelfPermission(permission) == PackageManager.PERMISSION_GRANTED
        }

    private fun requestCameraPermissions() {
        val activity = getCurrentActivity()
        if (activity !is PermissionAwareActivity) {
            rejectAndClear("E_PERMISSION_UNAVAILABLE", "Android camera permission cannot be requested.")
            return
        }

        activity.requestPermissions(
            requiredCameraPermissions().toTypedArray(),
            CAMERA_PERMISSION_REQUEST,
            PermissionListener { requestCode, _, grantResults ->
                if (requestCode != CAMERA_PERMISSION_REQUEST) return@PermissionListener false
                val granted = grantResults.isNotEmpty() && grantResults.all { it == PackageManager.PERMISSION_GRANTED }
                if (granted) {
                    launchCamera()
                } else {
                    rejectAndClear("E_CAMERA_PERMISSION", "Camera permission denied.")
                }
                true
            }
        )
    }

    private fun requiredCameraPermissions(): List<String> {
        val permissions = mutableListOf(Manifest.permission.CAMERA)
        if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.P) {
            permissions.add(Manifest.permission.WRITE_EXTERNAL_STORAGE)
        }
        return permissions
    }

    private fun filePayload(uri: Uri, fallbackName: String): WritableMap {
        val resolver = reactContext.contentResolver
        var name = fallbackName
        var size: Long? = null
        resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) {
                val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                if (nameIndex >= 0 && !cursor.isNull(nameIndex)) {
                    name = cursor.getString(nameIndex)
                }
                val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                if (sizeIndex >= 0 && !cursor.isNull(sizeIndex)) {
                    size = cursor.getLong(sizeIndex)
                }
            }
        }

        return Arguments.createMap().apply {
            putString("uri", uri.toString())
            putString("name", name)
            putString("mimeType", resolver.getType(uri) ?: "application/octet-stream")
            size?.let { putDouble("sizeBytes", it.toDouble()) }
        }
    }

    private fun beginRequest(promise: Promise): Boolean {
        if (pendingPromise != null) {
            promise.reject("E_CAPTURE_BUSY", "Another Android capture request is already active.")
            return false
        }
        pendingPromise = promise
        return true
    }

    private fun resolveAndClear(value: WritableMap) {
        pendingPhotoUri = null
        pendingPromise?.resolve(value)
        pendingPromise = null
    }

    private fun rejectAndClear(code: String, message: String, error: Throwable? = null) {
        pendingPhotoUri = null
        pendingPromise?.reject(code, message, error)
        pendingPromise = null
    }

    companion object {
        private const val PICK_FILE_REQUEST = 7101
        private const val CAPTURE_PHOTO_REQUEST = 7102
        private const val CAMERA_PERMISSION_REQUEST = 7103
        private val ACCEPTED_MIME_TYPES = arrayOf("application/pdf", "image/jpeg", "image/png", "text/plain")
    }
}
