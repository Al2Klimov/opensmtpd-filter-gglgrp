use itertools::Itertools;
use mail_parser::MessageParser;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::{stderr, stdin, stdout, Write};

fn main() -> std::io::Result<()> {
    let mut std_out = stdout().lock();
    let mut std_err = stderr().lock();
    let mut sessions = HashMap::<String, String>::new();

    for l in stdin().lines() {
        let line = l?;
        let mut fields = line.split("|");

        match fields.next() {
            None => {}
            Some(stream) => {
                match stream {
                    "config" => match fields.next() {
                        None => {}
                        Some(key) => {
                            if key == "ready" {
                                writeln!(std_out, "register|report|smtp-in|tx-data")?;
                                writeln!(std_out, "register|filter|smtp-in|data-line")?;
                                writeln!(std_out, "register|filter|smtp-in|commit")?;
                                writeln!(std_out, "register|report|smtp-in|link-disconnect")?;
                                writeln!(std_out, "register|ready")?;
                            }
                        }
                    },
                    "report" => {
                        fields.next(); // protocol version
                        fields.next(); // timestamp
                        fields.next(); // subsystem

                        match (fields.next(), fields.next()) {
                            (Some(phase), Some(session)) => match phase {
                                "tx-data" => {
                                    sessions.insert(session.to_owned(), String::new());
                                }
                                "link-disconnect" => {
                                    sessions.remove(session);
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    "filter" => {
                        fields.next(); // protocol version
                        fields.next(); // timestamp
                        fields.next(); // subsystem

                        match (fields.next(), fields.next(), fields.next()) {
                            (Some(phase), Some(session), Some(token)) => match phase {
                                "data-line" => {
                                    writeln!(
                                        std_out,
                                        "filter-dataline|{}|{}|{}",
                                        session,
                                        token,
                                        fields.clone().format("|")
                                    )?;

                                    let mut flds = fields.clone();

                                    match (flds.next(), flds.next()) {
                                        (Some("."), None) => {}
                                        _ => match sessions.get_mut(session) {
                                            None => {}
                                            Some(sessn) => {
                                                writeln!(sessn, "{}", fields.format("|")).unwrap();
                                            }
                                        },
                                    }
                                }
                                "commit" => {
                                    writeln!(
                                        std_out,
                                        "filter-result|{}|{}|{}",
                                        session,
                                        token,
                                        if match sessions.get(session) {
                                            None => true,
                                            Some(mail) =>
                                                match MessageParser::new().parse_headers(mail) {
                                                    None => {
                                                        writeln!(std_err, "Malformed eMail:")?;
                                                        write!(std_err, "{}", mail)?;
                                                        writeln!(std_err, ".")?;
                                                        true
                                                    }
                                                    Some(mail) =>
                                                        match mail.header("X-Google-Group-Id") {
                                                            None => true,
                                                            Some(_) => {
                                                                writeln!(
                                                                    std_err,
                                                                    "Google Group detected"
                                                                )?;
                                                                false
                                                            }
                                                        },
                                                },
                                        } {
                                            writeln!(std_err, "Allowing")?;
                                            "proceed"
                                        } else {
                                            writeln!(std_err, "Denying")?;
                                            "reject|550 Forbidden"
                                        }
                                    )?;
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
