// Archives of several files (PowerPoint, packaged imports) need a deflater;
// the browser build does not bundle one yet.
const missing = (): never => {
  throw new Error('zlib is not available in the browser build yet');
};
export const deflateRawSync = missing;
export const inflateRawSync = missing;
export const gunzipSync = missing;
export const gzipSync = missing;
