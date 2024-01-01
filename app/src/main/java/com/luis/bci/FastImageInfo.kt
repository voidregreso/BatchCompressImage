package com.luis.bci

import java.io.*

class FastImageInfo {
    var height = -1
    var width = -1
    var mimeType: String? = null

    constructor(file: File?) : this(FileInputStream(file))
    constructor(bytes: ByteArray?) : this(ByteArrayInputStream(bytes))
    constructor(istr: InputStream) {processStream(istr)}

    @Throws(IOException::class)
    private fun processStream(istr: InputStream) {
        val c1 = istr.read()
        val c2 = istr.read()
        var c3 = istr.read()

        mimeType = when {
            isGif(c1, c2, c3) -> {
                istr.skip(3)
                width = istr.readInt(2, false)
                height = istr.readInt(2, false)
                "image/gif"
            }
            isJpg(c1, c2) -> handleJpg(istr, c3)
            isPng(c1, c2, c3) -> {
                istr.skip(15)
                width = istr.readInt(2, true)
                istr.skip(2)
                height = istr.readInt(2, true)
                "image/png"
            }
            isBmp(c1, c2) -> {
                istr.skip(15)
                width = istr.readInt(2, false)
                istr.skip(2)
                height = istr.readInt(2, false)
                "image/bmp"
            }
            isWebP(c1, c2, c3) -> {
                val bytes = ByteArray(27)
                istr.read(bytes)
                width = bytes.decodeWebPWidth()
                height = bytes.decodeWebPHeight()
                "image/webp"
            }
            else -> handleOtherFormats(istr, c1, c2, c3)
        } ?: throw IOException("Unsupported image type")
    }

    private fun handleJpg(istr: InputStream, c3: Int): String {
        var currentByte = c3
        while (currentByte == 255) {
            val marker = istr.read()
            val len = istr.readInt(2, true)
            if (marker in listOf(192, 193, 194)) {
                istr.skip(1)
                height = istr.readInt(2, true)
                width = istr.readInt(2, true)
                return "image/jpeg"
            }
            istr.skip((len - 2).toLong())
            currentByte = istr.read()
        }
        return "image/jpeg"
    }

    private fun handleOtherFormats(istr: InputStream, c1: Int, c2: Int, c3: Int): String? {
        val c4 = istr.read()
        if (isTiff(c1, c2, c3, c4)) {
            val bigEndian = c1 == 'M'.code
            val ifd = istr.readInt(4, bigEndian)
            istr.skip((ifd - 8).toLong())
            val entries = istr.readInt(2, bigEndian)
            for (i in 1..entries) {
                val tag = istr.readInt(2, bigEndian)
                val fieldType = istr.readInt(2, bigEndian)
                val valOffset = if (fieldType in listOf(3, 8)) {
                    istr.readInt(2, bigEndian).also { istr.skip(2) }
                } else {
                    istr.readInt(4, bigEndian)
                }
                when (tag) {
                    256 -> width = valOffset
                    257 -> height = valOffset
                }
                if (width != -1 && height != -1) return "image/tiff"
            }
        }
        return null
    }

    override fun toString(): String {
        return "MIME Type : $mimeType\t Width : $width\t Height : $height"
    }
}

private fun InputStream.readInt(noOfBytes: Int, bigEndian: Boolean): Int {
    var ret = 0
    var sv = if (bigEndian) (noOfBytes - 1) * 8 else 0
    val cnt = if (bigEndian) -8 else 8
    for (i in 0 until noOfBytes) {
        ret = ret or (read() shl sv)
        sv += cnt
    }
    return ret
}

private fun ByteArray.decodeWebPWidth() = this[24].toInt() and 0xff shl 8 or (this[23].toInt() and 0xff)
private fun ByteArray.decodeWebPHeight() = this[26].toInt() and 0xff shl 8 or (this[25].toInt() and 0xff)

private fun isGif(c1: Int, c2: Int, c3: Int) = c1 == 'G'.code && c2 == 'I'.code && c3 == 'F'.code
private fun isJpg(c1: Int, c2: Int) = c1 == 0xFF && c2 == 0xD8
private fun isPng(c1: Int, c2: Int, c3: Int) = c1 == 137 && c2 == 80 && c3 == 78
private fun isBmp(c1: Int, c2: Int) = c1 == 66 && c2 == 77
private fun isWebP(c1: Int, c2: Int, c3: Int) = c1 == 'R'.code && c2 == 'I'.code && c3 == 'F'.code
private fun isTiff(c1: Int, c2: Int, c3: Int, c4: Int) = 
    (c1 == 'M'.code && c2 == 'M'.code && c3 == 0 && c4 == 42) || (c1 == 'I'.code && c2 == 'I'.code && c3 == 42 && c4 == 0)