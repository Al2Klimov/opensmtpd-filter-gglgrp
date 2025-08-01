mod util;

use mail_parser::MessageParser;
use std::collections::HashMap;
use std::io::{BufRead, Write, stderr, stdin, stdout};
use util::join_write_bytes;

fn main() -> std::io::Result<()> {
    let mut std_in = stdin().lock();
    let mut std_out = stdout().lock();
    let mut std_err = stderr().lock();

    let mut line = Vec::<u8>::new();
    let mut sessions = HashMap::<Vec<u8>, Vec<u8>>::new();

    loop {
        line.clear();
        std_in.read_until(b'\n', &mut line)?;

        while line
            .pop_if(|last| match last {
                b'\r' => true,
                b'\n' => true,
                _ => false,
            })
            .is_some()
        {}

        let mut fields = line.split(|&sep| sep == b'|');

        match fields.next() {
            None => {}
            Some(stream) => {
                match stream {
                    b"config" => match fields.next() {
                        None => {}
                        Some(key) => {
                            if key == b"ready" {
                                writeln!(std_out, "register|report|smtp-in|tx-data")?;
                                writeln!(std_out, "register|filter|smtp-in|data-line")?;
                                writeln!(std_out, "register|filter|smtp-in|commit")?;
                                writeln!(std_out, "register|report|smtp-in|link-disconnect")?;
                                writeln!(std_out, "register|ready")?;
                            }
                        }
                    },
                    b"report" => {
                        fields.next(); // protocol version
                        fields.next(); // timestamp
                        fields.next(); // subsystem

                        match (fields.next(), fields.next()) {
                            (Some(phase), Some(session)) => match phase {
                                b"tx-data" => {
                                    sessions.insert(session.to_owned(), Vec::<u8>::new());
                                }
                                b"link-disconnect" => {
                                    sessions.remove(session);
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    b"filter" => {
                        fields.next(); // protocol version
                        fields.next(); // timestamp
                        fields.next(); // subsystem

                        match (fields.next(), fields.next(), fields.next()) {
                            (Some(phase), Some(session), Some(token)) => match phase {
                                b"data-line" => {
                                    std_out.write_all(b"filter-dataline|")?;
                                    std_out.write_all(session)?;
                                    std_out.write_all(b"|")?;
                                    std_out.write_all(token)?;
                                    std_out.write_all(b"|")?;

                                    join_write_bytes(&mut std_out, b"|", fields.clone())?;
                                    writeln!(std_out, "")?;

                                    let mut flds = fields.clone();

                                    match (flds.next(), flds.next()) {
                                        (Some(b"."), None) => {}
                                        _ => match sessions.get_mut(session) {
                                            None => {}
                                            Some(sessn) => {
                                                join_write_bytes(sessn, b"|", fields)?;
                                                writeln!(sessn, "")?;
                                            }
                                        },
                                    }
                                }
                                b"commit" => {
                                    std_out.write_all(b"filter-result|")?;
                                    std_out.write_all(session)?;
                                    std_out.write_all(b"|")?;
                                    std_out.write_all(token)?;

                                    writeln!(
                                        std_out,
                                        "|{}",
                                        if match sessions.get(session) {
                                            None => true,
                                            Some(mail) =>
                                                match MessageParser::new().parse_headers(mail) {
                                                    None => {
                                                        writeln!(std_err, "Malformed eMail:")?;
                                                        std_err.write_all(mail)?;
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
}
