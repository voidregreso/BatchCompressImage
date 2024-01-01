package com.luis.bci

import android.app.Activity
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.*
import android.provider.Settings
import android.util.Log
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.core.app.ActivityCompat
import androidx.documentfile.provider.DocumentFile
import kotlinx.coroutines.*
import java.io.FileOutputStream
import java.nio.ByteBuffer

class MainActivity : Activity() {
    private val fileRequestCode = 0xFFFF
    private val storagePermissionCode = 0xFFFE
    private val compRate = 82

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (Build.VERSION.SDK_INT >= 30) {
            if (!Environment.isExternalStorageManager()) {
                val intent = Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION)
                startActivityForResult(intent, storagePermissionCode)
                return
            }
            openDocumentSelector()
        } else {
            ActivityCompat.requestPermissions(
                this,
                arrayOf(
                    android.Manifest.permission.READ_EXTERNAL_STORAGE,
                    android.Manifest.permission.WRITE_EXTERNAL_STORAGE
                ),
                storagePermissionCode
            )
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == storagePermissionCode) {
            if (Build.VERSION.SDK_INT >= 30 && Environment.isExternalStorageManager()) {
                openDocumentSelector()
            } else {
                Toast.makeText(this, "Permission Denied", Toast.LENGTH_SHORT).show()
                finish()
            }
            return
        }

        if (requestCode == fileRequestCode && resultCode == RESULT_OK) {
            data?.data?.let { uri ->
                DocumentFile.fromTreeUri(this, uri)?.also { documentFile ->
                    val alertDialog = AlertDialog.Builder(this)
                        .setMessage("Processing folder ${documentFile.name} ...")
                        .setCancelable(false)
                        .create()

                    alertDialog.show()
                    val job = Job()
                    val scope = CoroutineScope(job)
                    scope.launch(Dispatchers.IO) {
                        processFiles(documentFile)
                        withContext(Dispatchers.Main) {
                            alertDialog.dismiss()
                            Toast.makeText(applicationContext, "Completed", Toast.LENGTH_LONG).show()
                            finish()
                        }
                    }
                }
            }
        } else {
            finish()
        }
    }

    private fun pngIsTransparent(bmp: Bitmap) : Boolean {
        for (y in 0 until bmp.height) {
            for(x in 0 until bmp.width) {
                val pix = bmp.getPixel(x, y).toLong()
                // Found transparency
                if((pix and 0xff000000) != 0xff000000) return true
            }
        }
        return false
    }

    private suspend fun processFiles(folder: DocumentFile) = withContext(Dispatchers.IO) {
        folder.listFiles().map { file ->
            async {
                val fileName = file.name ?: return@async
                val fileExtension = fileName.substringAfterLast('.', "").lowercase()

                try {
                    when (fileExtension) {
                        "png", "bmp", "webp" -> processImageFile(file, fileExtension, folder)
                        "jpg", "jpeg" -> processJpgFile(file)
                    }
                } catch (e: Exception) {
                    Log.e("ImageProcessing", "Error processing file: $fileName", e)
                }
            }
        }.awaitAll()  // Wait for all co-routine to complete
    }

    private fun processImageFile(file: DocumentFile, fileExtension: String, origFolder: DocumentFile) {
        contentResolver.openInputStream(file.uri)?.use { inputStream ->
            BitmapFactory.decodeStream(inputStream)?.also { bitmap ->
                if (fileExtension != "png" || !pngIsTransparent(bitmap)) {
                    file.name?.let {
                        createConvertedJPEG(file, origFolder,
                            it, fileExtension, bitmap)
                    }
                }
            }
        }
    }

    private fun createConvertedJPEG(file: DocumentFile, documentFile: DocumentFile, fileName: String, fileExtension: String, bitmap: Bitmap) {
        val newFile = documentFile.createFile("image/jpeg", fileName.removeSuffix(".$fileExtension") + ".jpg")
        newFile?.uri?.let { newUri ->
            contentResolver.openOutputStream(newUri)?.use { outputStream ->
                if (bitmap.compress(Bitmap.CompressFormat.JPEG, compRate, outputStream)) {
                    file.delete()
                }
            }
        }
    }

    private fun processJpgFile(file: DocumentFile) {
        contentResolver.openInputStream(file.uri)?.use { inputStream ->
            // Read all bytes from the inputStream at once
            val data = inputStream.readBytes()
            // Now you can process this data
            val imgInfo = FastImageInfo(data)
            compressAndReplaceJPG(file, data, imgInfo)
        }
    }

    private fun compressAndReplaceJPG(file: DocumentFile, data: ByteArray, imgInfo: FastImageInfo) {
        val csp = CCSParameter(true, compRate, CCSParameter.ChromaSubsampling.Auto,
            compRate, false, compRate, true, imgInfo.width, imgInfo.height)
        CaesiumNative.compressPic(data, csp)?.let { compressedData ->
            contentResolver.openFileDescriptor(file.uri, "rw")?.use { parcelFileDescriptor ->
                val fileDescriptor = parcelFileDescriptor.fileDescriptor
                FileOutputStream(fileDescriptor).channel.use { fileChannel ->
                    fileChannel.truncate(0) // Purge file content
                    fileChannel.write(ByteBuffer.wrap(compressedData))
                }
            }
        }
    }

    private fun openDocumentSelector() {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE)
        startActivityForResult(intent, fileRequestCode)
    }

    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == storagePermissionCode && grantResults.isNotEmpty() && grantResults[0] == 0) {
            openDocumentSelector()
        } else {
            Toast.makeText(this, "Permission Denied", Toast.LENGTH_SHORT).show()
            finish()
        }
    }
}
