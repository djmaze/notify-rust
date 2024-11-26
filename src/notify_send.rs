use std::ops::{Deref, DerefMut};
use std::process::{Command, Stdio};

use crate::Timeout;
use crate::{error::Result, Notification};

#[derive(Clone, Debug)]
pub struct NotificationHandle {
    id: u32,
    notification: Notification,
}

impl NotificationHandle {
    pub fn update_fallible(&mut self) -> Result<()> {
        self.id = send_notification(&self.notification, Some(self.id))?;
        Ok(())
    }

    pub fn update(&mut self) {
        self.update_fallible().expect("Could not send notification");
    }

    /// Returns the Handle's id.
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Deref for NotificationHandle {
    type Target = Notification;

    fn deref(&self) -> &Notification {
        &self.notification
    }
}

impl DerefMut for NotificationHandle {
    fn deref_mut(&mut self) -> &mut Notification {
        &mut self.notification
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
#[cfg(all(feature = "notify_send"))]
pub(crate) fn show_notification(notification: &Notification) -> Result<NotificationHandle> {
    let id = send_notification(notification, None)?;

    Ok(NotificationHandle {
        id,
        notification: notification.clone(),
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
#[cfg(all(feature = "async", feature = "notify_send"))]
pub(crate) async fn show_notification_async(
    notification: &Notification,
) -> Result<NotificationHandle> {
    async move { show_notification(&notification) }.await
}

fn send_notification(notification: &Notification, previous_id_option: Option<u32>) -> Result<u32> {
    let mut command = Command::new("notify-send");

    // TODO hints
    // TODO actions

    if let Timeout::Milliseconds(secs) = notification.timeout {
        command.args(["--expire-time", &format!("{secs:}")]);
    };

    if let Some(previous_id) = previous_id_option {
        command.args(["--replace-id", &previous_id.to_string()]);
    };

    command.args([
        "--print-id",
        "--app-name",
        &notification.appname,
        "--icon",
        &notification.icon,
        &notification.summary,
        &notification.body,
    ]);

    let output = command.stderr(Stdio::inherit()).output()?;
    let string = String::from_utf8(output.stdout)?;
    Ok(string.trim_end().parse::<u32>()?)
}
