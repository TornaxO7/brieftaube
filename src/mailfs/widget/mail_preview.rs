use crate::backend::mails::types::MailAddress;
use throbber_widgets_tui::ThrobberState;

#[derive(Debug)]
pub enum MailPreview<'a> {
    Loading(&'a mut ThrobberState),
    Loaded {
        from: String,
        to: String,
        cc: String,
        subject: String,
        preview: String,
        received_at: String,
    },
}

// impl From<(&MailData, ThreadMarker)> for MailPreview {
//     fn from(data: (&MailData, ThreadMarker)) -> Self {
//         let mail = data.0;
//         let thread_marker = data.1;

//         let id = mail.id.clone();
//         let from = addresses_to_string(&mail.from);
//         let to = addresses_to_string(&mail.to);
//         let cc = addresses_to_string(&mail.cc);

//         let subject = mail.subject.clone();
//         let preview = mail.preview.clone();
//         let received_at = mail
//             .received_at
//             .format("%a, %e %b %Y, %H:%M:%S")
//             .to_string();
//         let has_attachment = mail.has_attachment;
//         let keywords = mail.keywords.clone();

//         Self {
//             id,
//             from,
//             to,
//             cc,
//             subject,
//             preview,
//             received_at,
//             has_attachment,
//             keywords,
//             thread_marker,
//         }
//     }
// }

pub fn addresses_to_string(addresses: &[MailAddress]) -> String {
    let mut iterator = addresses.iter();
    let first = iterator
        .next()
        .map(|addr| format!("{}", addr))
        .unwrap_or(String::new());

    iterator.fold(first, |acc, addr| format!("{acc}, {}", addr.to_string()))
}
