use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use melib::{
    mbox::{Length, MboxFormat, MessageIterator, Offset},
    Envelope, EnvelopeHash,
};

#[derive(PartialEq, Eq, Hash)]
struct Sender {
    display_name: String,
    email: String,
}

type MailFrom = HashMap<Sender, i32>;

fn main() {
    let file_path = env::args().nth(1).expect("exported mbox file");

    let index: Arc<Mutex<HashMap<EnvelopeHash, (Offset, Length)>>> =
        Arc::new(Mutex::new(HashMap::default()));
    let mut mails = MailFrom::new();

    std::fs::read_to_string(file_path)
        .expect("failed to read file")
        .split("\nFrom ")
        .enumerate()
        // HACK: workround for gmail exported mbox file
        .map(|(idx, mail)| {
            if idx > 0 {
                format!("From {mail}")
            } else {
                mail.to_string()
            }
        })
        .for_each(|file_content| {
            let message_iter = MessageIterator {
                index: index.clone(),
                input: file_content.as_bytes(),
                offset: 0,
                file_offset: 0,
                format: Some(MboxFormat::MboxCl),
            };
            let Ok(envelopes) = message_iter.collect::<Result<Vec<Envelope>, _>>() else {return};
            assert_eq!(envelopes.len(), 1);

            for envelop in envelopes {
                let addrs = envelop.from();
                for addr in addrs {
                    let display_name = addr
                        .get_display_name()
                        .unwrap_or("<no_display_name>".to_owned());
                    let email = addr.get_email();

                    *mails
                        .entry(Sender {
                            display_name,
                            email,
                        })
                        .or_insert(0) += 1;
                }
            }
        });
    let mut mails = mails.into_iter().collect::<Vec<(Sender, i32)>>();
    mails.sort_by_key(|item| -item.1);

    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    wtr.write_record(&["display name", "email", "count"])
        .unwrap();

    for (
        Sender {
            display_name,
            email,
        },
        count,
    ) in mails.iter()
    {
        wtr.write_record(&[display_name, email, &count.to_string()])
            .unwrap();
    }

    let contents = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

    std::fs::write("result.csv", contents).unwrap();
}
