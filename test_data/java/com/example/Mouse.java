package com.example;

public class Mouse extends IODevice implements IODevice.Character {
    @Override 
    public Integer getMajorNumber() {
        return 13;
    }

    @Override
    public Integer getMinorNumber() {
        return 76;
    } 

    public int getBufferSize() {
        return 4096;
    } 
}
