import { normalizePickedFile } from '../native/capture';

describe('capture native adapter', () => {
  it('normalizes native file payloads for upload drafts', () => {
    expect(
      normalizePickedFile({
        uri: 'content://office/memo.pdf',
        name: 'memo.pdf',
        mimeType: 'application/pdf',
        sizeBytes: 4096
      })
    ).toEqual({
      uri: 'content://office/memo.pdf',
      name: 'memo.pdf',
      type: 'application/pdf',
      sizeBytes: 4096
    });
  });
});
