// SPDX-License-Identifier: MIT OR Apache-2.0
package com.duidev.craftmobile;

import com.google.androidgamesdk.GameActivity;
import android.content.*;
import android.database.Cursor;
import android.graphics.*;
import android.media.ExifInterface;
import android.net.Uri;
import android.os.*;
import android.provider.*;
import org.json.JSONObject;
import java.io.*;
import java.util.*;
import java.util.concurrent.*;

/** URI grants stay in Android; Rust only receives app-private copies. */
public final class CraftActivity extends GameActivity {
    static { System.loadLibrary("craft"); }
    @Override protected void onCreate(Bundle state) {
        super.onCreate(state);
        androidx.core.view.WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
        android.view.View content = findViewById(android.R.id.content);
        androidx.core.view.ViewCompat.setOnApplyWindowInsetsListener(content, (view, insets) -> {
            androidx.core.graphics.Insets bars = insets.getInsets(androidx.core.view.WindowInsetsCompat.Type.systemBars() | androidx.core.view.WindowInsetsCompat.Type.displayCutout());
            view.setPadding(bars.left, bars.top, bars.right, bars.bottom);
            return insets;
        });
        androidx.core.view.ViewCompat.requestApplyInsets(content);
    }
    private final ExecutorService worker = Executors.newSingleThreadExecutor();
    private boolean busy;
    private String result;
    private long operation;
    private long pendingToken;
    private String exportPath;
    private static final int OPEN = 201, PHOTO = 202, EXPORT = 203;
    private synchronized long claim() {
        if (busy || result != null) return 0;
        busy = true;
        return ++operation;
    }
    private synchronized void finish(long token, String path, String error) {
        if (!busy || token != operation) return;
        JSONObject value = new JSONObject();
        try { if (path != null) value.put("path", path); if (error != null) value.put("error", error); }
        catch (org.json.JSONException impossible) { /* Keys and values are fixed strings. */ }
        result = value.toString(); busy = false;
    }
    public synchronized String pollResult() { String value = result; result = null; return value; }
    public String documentsPath() { return new File(getFilesDir(), "Documents").getAbsolutePath(); }
    private File local(String path) throws IOException {
        File root = new File(documentsPath()).getCanonicalFile();
        File file = new File(path).getCanonicalFile();
        if (!file.getPath().startsWith(root.getPath() + File.separator) || !file.isFile())
            throw new IOException("Choose a saved file inside this application's Documents folder");
        return file;
    }
    public boolean openDocuments() { return launch(OPEN, null); }
    public boolean openPhotos() { return launch(PHOTO, null); }
    public boolean exportFile(String path) { return launch(EXPORT, path); }
    private boolean launch(int mode, String path) {
        long token = claim(); if (token == 0) return false;
        runOnUiThread(() -> {
            try {
                Intent intent;
                if (mode == EXPORT) {
                    File file = local(path);
                    intent = new Intent(Intent.ACTION_CREATE_DOCUMENT).setType("application/octet-stream")
                        .addCategory(Intent.CATEGORY_OPENABLE).putExtra(Intent.EXTRA_TITLE, file.getName());
                    exportPath = file.getAbsolutePath();
                } else if (mode == PHOTO && Build.VERSION.SDK_INT >= 33) {
                    intent = new Intent(MediaStore.ACTION_PICK_IMAGES).setType("image/*");
                } else {
                    intent = new Intent(mode == PHOTO ? Intent.ACTION_GET_CONTENT : Intent.ACTION_OPEN_DOCUMENT)
                        .setType(mode == PHOTO ? "image/*" : "*/*").addCategory(Intent.CATEGORY_OPENABLE);
                }
                intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
                pendingToken = token;
                startActivityForResult(intent, mode);
            } catch (Exception e) { finish(token, null, "The system picker could not be opened: " + e.getMessage()); }
        });
        return true;
    }
    private File importFolder() throws IOException {
        File documents = new File(documentsPath()).getCanonicalFile();
        File imports = new File(documents, "Imports").getCanonicalFile();
        if (!imports.getPath().startsWith(documents.getPath() + File.separator)) throw new IOException("Invalid import folder");
        File folder = new File(imports, UUID.randomUUID().toString());
        if (!folder.mkdirs()) throw new IOException("Could not create import folder");
        return folder;
    }
    private static void copy(InputStream in, OutputStream out) throws IOException {
        if (in == null || out == null) throw new IOException("The selected provider did not supply a stream");
        byte[] buffer = new byte[65536]; long count = 0; int read;
        while ((read = in.read(buffer)) != -1) {
            count += read;
            if (count > 512L * 1024 * 1024) throw new IOException("Choose a file smaller than 512 MB");
            out.write(buffer, 0, read);
        }
    }
    private String displayName(Uri uri) {
        try (Cursor cursor = getContentResolver().query(uri, new String[]{OpenableColumns.DISPLAY_NAME}, null, null, null)) {
            if (cursor != null && cursor.moveToFirst()) {
                String name = cursor.getString(0);
                if (name != null) {
                    name = name.replaceAll("[\\\\/\\p{Cntrl}]", "_");
                    if (!name.equals(".") && !name.equals("..") && !name.isEmpty() && name.length() < 200) return name;
                }
            }
        } catch (Exception ignored) { }
        return "Imported";
    }
    private File normalize(File original, File folder) throws IOException {
        BitmapFactory.Options bounds = new BitmapFactory.Options(); bounds.inJustDecodeBounds = true;
        BitmapFactory.decodeFile(original.getPath(), bounds);
        if (bounds.outWidth <= 0 || bounds.outHeight <= 0) throw new IOException("The selected photo could not be decoded");
        BitmapFactory.Options options = new BitmapFactory.Options(); options.inSampleSize = 1;
        while (Math.max(bounds.outWidth, bounds.outHeight) / options.inSampleSize > 8192) options.inSampleSize *= 2;
        Bitmap image = BitmapFactory.decodeFile(original.getPath(), options);
        if (image == null) throw new IOException("The selected photo could not be decoded");
        Bitmap oriented = image;
        try {
            int orientation = new ExifInterface(original.getPath()).getAttributeInt(ExifInterface.TAG_ORIENTATION, 1);
            Matrix matrix = new Matrix();
            switch (orientation) {
                case 2: matrix.setScale(-1, 1); break;
                case 3: matrix.setRotate(180); break;
                case 4: matrix.setScale(1, -1); break;
                case 5: matrix.setRotate(90); matrix.postScale(-1, 1); break;
                case 6: matrix.setRotate(90); break;
                case 7: matrix.setRotate(270); matrix.postScale(-1, 1); break;
                case 8: matrix.setRotate(270); break;
            }
            if (!matrix.isIdentity()) oriented = Bitmap.createBitmap(image, 0, 0, image.getWidth(), image.getHeight(), matrix, true);
            File png = new File(folder, "Photo.png");
            try (OutputStream out = new FileOutputStream(png)) {
                if (!oriented.compress(Bitmap.CompressFormat.PNG, 100, out)) throw new IOException("PNG conversion failed");
            }
            if (png.length() > 64L * 1024 * 1024) throw new IOException("Choose an image smaller than 64 MB as PNG");
            return png;
        } finally { if (oriented != image) oriented.recycle(); image.recycle(); }
    }
    @Override public void onActivityResult(int request, int code, Intent data) {
        super.onActivityResult(request, code, data);
        if (request != OPEN && request != PHOTO && request != EXPORT) return;
        long token = pendingToken;
        if (code != RESULT_OK || data == null || data.getData() == null) { finish(token, null, null); return; }
        Uri uri = data.getData(); String source = exportPath;
        worker.execute(() -> {
            File folder = null;
            try {
                if (request == EXPORT) {
                    try (InputStream in = new FileInputStream(local(source)); OutputStream out = getContentResolver().openOutputStream(uri, "wt")) { copy(in, out); }
                    finish(token, source, null); return;
                }
                folder = importFolder();
                File imported = new File(folder, request == PHOTO ? "original-photo" : displayName(uri));
                try (InputStream in = getContentResolver().openInputStream(uri); OutputStream out = new FileOutputStream(imported)) { copy(in, out); }
                if (request == PHOTO) { File png = normalize(imported, folder); imported.delete(); imported = png; }
                finish(token, imported.getAbsolutePath(), null);
            } catch (Exception | OutOfMemoryError e) {
                if (folder != null) { File[] files = folder.listFiles(); if (files != null) for (File file : files) file.delete(); folder.delete(); }
                finish(token, null, "Import/export failed: " + e.getMessage());
            }
        });
    }
    public boolean savePhoto(String path) {
        long token = claim(); if (token == 0) return false;
        worker.execute(() -> {
            Uri created = null;
            try {
                File file = local(path); String name = file.getName().toLowerCase(Locale.ROOT);
                String mime = name.endsWith(".png") ? "image/png" : (name.endsWith(".jpg") || name.endsWith(".jpeg")) ? "image/jpeg" :
                    (name.endsWith(".heic") || name.endsWith(".heif")) ? "image/heic" : null;
                if (mime == null) throw new IOException("Export a PNG, JPEG or HEIC image before saving to Photos");
                BitmapFactory.Options check = new BitmapFactory.Options(); check.inJustDecodeBounds = true;
                BitmapFactory.decodeFile(file.getPath(), check);
                if (check.outWidth <= 0) throw new IOException("The saved file is not a readable image");
                ContentValues values = new ContentValues(); values.put(MediaStore.Images.Media.DISPLAY_NAME, file.getName());
                values.put(MediaStore.Images.Media.MIME_TYPE, mime); values.put(MediaStore.Images.Media.RELATIVE_PATH, Environment.DIRECTORY_PICTURES + "/CraftMobile");
                values.put(MediaStore.Images.Media.IS_PENDING, 1);
                created = getContentResolver().insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values);
                if (created == null) throw new IOException("The gallery could not create an image");
                try (InputStream in = new FileInputStream(file); OutputStream out = getContentResolver().openOutputStream(created)) { copy(in, out); }
                values.clear(); values.put(MediaStore.Images.Media.IS_PENDING, 0); getContentResolver().update(created, values, null, null);
                finish(token, file.getPath(), null);
            } catch (Exception e) {
                if (created != null) try { getContentResolver().delete(created, null, null); } catch (Exception ignored) { }
                finish(token, null, "Saving to gallery failed: " + e.getMessage());
            }
        });
        return true;
    }
    @Override protected void onDestroy() { worker.shutdown(); super.onDestroy(); }
}
