// Automatically generated by flapigen
package army.warfare.skiter;


public final class PaintLayer {

    public PaintLayer(long element, boolean is_foreground) {
        mNativeObj = init(element, is_foreground);
    }
    private static native long init(long element, boolean is_foreground);

    public synchronized void delete() {
        if (mNativeObj != 0) {
            do_delete(mNativeObj);
            mNativeObj = 0;
       }
    }
    @Override
    protected void finalize() throws Throwable {
        try {
            delete();
        }
        finally {
             super.finalize();
        }
    }
    private static native void do_delete(long me);
    /*package*/ PaintLayer(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}