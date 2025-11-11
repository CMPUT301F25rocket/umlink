package com.example;

import java.util.List;
import java.util.ArrayList;

public class Keyboard extends IODevice implements IODevice.Character {
    @UmlAggregate(selfCard = "1", label="contains", otherCard="0..*")
    List<KeyCode> keys;

    public Keyboard() {
        keys = new ArrayList<KeyCode>();
    }

    public List<KeyCode> getKeys() {
        return keys;
    }

    public void setKeys(List<KeyCode> keys) {
        this.keys = keys;
    }

    public void press(KeyCode key) {
        return;
    }

    @Override 
    public Integer getMajorNumber() {
        return 13;
    }

    @Override
    public Integer getMinorNumber() {
        return 77;
    } 

    public int getBufferSize() {
        return 4096;
    } 
}
