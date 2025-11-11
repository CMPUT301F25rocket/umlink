package com.example;

public abstract class IODevice {
    public abstract Integer getMajorNumber();
    public abstract Integer getMinorNumber();

    public interface Character {
        public int getBufferSize();
    }

    public interface Network {
        public enum State {
            UP, DOWN
        }

        public String getMacAddress();
        public State getState();
    }

    @UmlSkip
    public interface Block {
        public String getMountPoint();
    }
}
