pub mod sync {
    use std::io::Read;

    pub fn to_vec(mut read: impl Read) -> Vec<u8> {
        let mut output = vec![];
        read.read_to_end(&mut output).unwrap();
        output
    }
}

#[cfg(feature = "futures-io")]
pub mod futures_io {
    pub mod bufread {
        use crate::utils::InputStream;
        use futures::{stream::TryStreamExt as _, AsyncBufRead};

        pub fn from(input: &InputStream) -> impl AsyncBufRead {
            // By using the stream here we ensure that each chunk will require a separate
            // read/poll_fill_buf call to process to help test reading multiple chunks.
            input.stream().into_async_read()
        }
    }

    pub mod read {
        use crate::utils::{block_on, pin_mut};
        use futures::io::{copy_buf, AsyncRead, AsyncReadExt, BufReader, Cursor};

        pub fn to_vec(read: impl AsyncRead) -> Vec<u8> {
            // TODO: https://github.com/rust-lang-nursery/futures-rs/issues/1510
            // All current test cases are < 100kB
            let mut output = Cursor::new(vec![0; 102_400]);
            pin_mut!(read);
            let len = block_on(copy_buf(BufReader::with_capacity(2, read), &mut output)).unwrap();
            let mut output = output.into_inner();
            output.truncate(len as usize);
            output
        }

        pub fn poll_read(reader: impl AsyncRead, output: &mut [u8]) -> std::io::Result<usize> {
            pin_mut!(reader);
            block_on(reader.read(output))
        }
    }

    pub mod write {
        use crate::utils::{block_on, Pin, TrackClosed};
        use futures::io::{AsyncWrite, AsyncWriteExt as _};
        use futures_test::io::AsyncWriteTestExt as _;

        pub fn to_vec(
            input: &[Vec<u8>],
            create_writer: impl for<'a> FnOnce(
                &'a mut (dyn AsyncWrite + Unpin),
            ) -> Pin<Box<dyn AsyncWrite + 'a>>,
            limit: usize,
        ) -> Vec<u8> {
            let mut output = Vec::new();
            {
                let mut test_writer = TrackClosed::new(
                    (&mut output)
                        .limited_write(limit)
                        .interleave_pending_write(),
                );
                {
                    let mut writer = create_writer(&mut test_writer);
                    for chunk in input {
                        block_on(writer.write_all(chunk)).unwrap();
                        block_on(writer.flush()).unwrap();
                    }
                    block_on(writer.close()).unwrap();
                }
                assert!(test_writer.is_closed());
            }
            output
        }
    }
}

#[cfg(feature = "stream")]
pub mod stream {
    use crate::utils::{block_on, pin_mut, Bytes, Result};
    use futures::stream::{Stream, TryStreamExt as _};

    pub fn to_vec(stream: impl Stream<Item = Result<Bytes>>) -> Vec<u8> {
        pin_mut!(stream);
        block_on(stream.try_collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .flatten()
            .collect()
    }
}

#[cfg(feature = "tokio-02")]
pub mod tokio_02 {
    pub mod bufread {
        use crate::utils::InputStream;
        use tokio_02::io::{stream_reader, AsyncBufRead};

        pub fn from(input: &InputStream) -> impl AsyncBufRead {
            // By using the stream here we ensure that each chunk will require a separate
            // read/poll_fill_buf call to process to help test reading multiple chunks.
            stream_reader(input.stream())
        }
    }

    pub mod read {
        use crate::utils::{block_on, pin_mut, tokio_02_ext::copy_buf};
        use std::io::Cursor;
        use tokio_02::io::{AsyncRead, AsyncReadExt, BufReader};

        pub fn to_vec(read: impl AsyncRead) -> Vec<u8> {
            let mut output = Cursor::new(vec![0; 102_400]);
            pin_mut!(read);
            let len = block_on(copy_buf(BufReader::with_capacity(2, read), &mut output)).unwrap();
            let mut output = output.into_inner();
            output.truncate(len as usize);
            output
        }

        pub fn poll_read(reader: impl AsyncRead, output: &mut [u8]) -> std::io::Result<usize> {
            pin_mut!(reader);
            block_on(reader.read(output))
        }
    }

    pub mod write {
        use crate::utils::{
            block_on, tokio_02_ext::AsyncWriteTestExt as _, track_closed::TrackClosed, Pin,
        };
        use std::io::Cursor;
        use tokio_02::io::{AsyncWrite, AsyncWriteExt as _};

        pub fn to_vec(
            input: &[Vec<u8>],
            create_writer: impl for<'a> FnOnce(
                &'a mut (dyn AsyncWrite + Unpin),
            ) -> Pin<Box<dyn AsyncWrite + 'a>>,
            limit: usize,
        ) -> Vec<u8> {
            let mut output = Cursor::new(Vec::new());
            {
                let mut test_writer = TrackClosed::new(
                    (&mut output)
                        .limited_write(limit)
                        .interleave_pending_write(),
                );
                {
                    let mut writer = create_writer(&mut test_writer);
                    for chunk in input {
                        block_on(writer.write_all(chunk)).unwrap();
                        block_on(writer.flush()).unwrap();
                    }
                    block_on(writer.shutdown()).unwrap();
                }
                assert!(test_writer.is_closed());
            }
            output.into_inner()
        }
    }
}

#[cfg(feature = "tokio-03")]
pub mod tokio_03 {
    pub mod bufread {
        use crate::utils::InputStream;
        use tokio_03::io::AsyncBufRead;
        use tokio_util::io::StreamReader;

        pub fn from(input: &InputStream) -> impl AsyncBufRead {
            // By using the stream here we ensure that each chunk will require a separate
            // read/poll_fill_buf call to process to help test reading multiple chunks.
            StreamReader::new(input.stream())
        }
    }

    pub mod read {
        use crate::utils::{block_on, pin_mut, tokio_03_ext::copy_buf};
        use std::io::Cursor;
        use tokio_03::io::{AsyncRead, AsyncReadExt, BufReader};

        pub fn to_vec(read: impl AsyncRead) -> Vec<u8> {
            let mut output = Cursor::new(vec![0; 102_400]);
            pin_mut!(read);
            let len = block_on(copy_buf(BufReader::with_capacity(2, read), &mut output)).unwrap();
            let mut output = output.into_inner();
            output.truncate(len as usize);
            output
        }

        pub fn poll_read(reader: impl AsyncRead, output: &mut [u8]) -> std::io::Result<usize> {
            pin_mut!(reader);
            block_on(reader.read(output))
        }
    }

    pub mod write {
        use crate::utils::{
            block_on, tokio_03_ext::AsyncWriteTestExt as _, track_closed::TrackClosed, Pin,
        };
        use std::io::Cursor;
        use tokio_03::io::{AsyncWrite, AsyncWriteExt as _};

        pub fn to_vec(
            input: &[Vec<u8>],
            create_writer: impl for<'a> FnOnce(
                &'a mut (dyn AsyncWrite + Unpin),
            ) -> Pin<Box<dyn AsyncWrite + 'a>>,
            limit: usize,
        ) -> Vec<u8> {
            let mut output = Cursor::new(Vec::new());
            {
                let mut test_writer = TrackClosed::new(
                    (&mut output)
                        .limited_write(limit)
                        .interleave_pending_write(),
                );
                {
                    let mut writer = create_writer(&mut test_writer);
                    for chunk in input {
                        block_on(writer.write_all(chunk)).unwrap();
                        block_on(writer.flush()).unwrap();
                    }
                    block_on(writer.shutdown()).unwrap();
                }
                assert!(test_writer.is_closed());
            }
            output.into_inner()
        }
    }
}
