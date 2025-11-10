package com.rocket.radar.notifications;

import java.util.Date;

/**
 * Stub class for testing umlink.
 * This is a simplified version extracted from an Android project.
 */
public class Notification {
    private String eventTitle;
    private String notificationType;
    private int image;
    private Date timestamp;
    private String notificationId;
    private String userNotificationId;
    private boolean readStatus;

    public Notification() {
    }

    public String getEventTitle() {
        return eventTitle;
    }

    public String getNotificationType() {
        return notificationType;
    }

    public boolean isReadStatus() {
        return readStatus;
    }

    public int getImage() {
        return image;
    }

    public Date getTimestamp() {
        return timestamp;
    }

    public String getNotificationId() {
        return notificationId;
    }

    public String getUserNotificationId() {
        return userNotificationId;
    }

    public void setReadStatus(boolean readStatus) {
        this.readStatus = readStatus;
    }

    public void setUserNotificationId(String userNotificationId) {
        this.userNotificationId = userNotificationId;
    }
}
