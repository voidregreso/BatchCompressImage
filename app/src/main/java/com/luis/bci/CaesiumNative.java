package com.luis.bci;

public class CaesiumNative {
	
	static {
		System.loadLibrary("caesium_jni");
	}
	public static native byte[] compressPic(byte[] inData, CCSParameter conf);

}