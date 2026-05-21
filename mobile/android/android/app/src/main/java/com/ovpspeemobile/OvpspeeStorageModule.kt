package com.ovpspeemobile

import android.content.Context
import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactContextBaseJavaModule
import com.facebook.react.bridge.ReactMethod

class OvpspeeStorageModule(private val reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {
    private val prefs =
        reactContext.getSharedPreferences("ovpspee_mobile_storage", Context.MODE_PRIVATE)

    override fun getName(): String = "OvpspeeStorage"

    @ReactMethod
    fun setItem(key: String, value: String, promise: Promise) {
        prefs.edit().putString(key, value).apply()
        promise.resolve(null)
    }

    @ReactMethod
    fun getItem(key: String, promise: Promise) {
        promise.resolve(prefs.getString(key, null))
    }

    @ReactMethod
    fun removeItem(key: String, promise: Promise) {
        prefs.edit().remove(key).apply()
        promise.resolve(null)
    }
}
