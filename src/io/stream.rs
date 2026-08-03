use std::fs::File;
use std::io::{PipeReader, PipeWriter, Write};
use std::process::Stdio;

pub enum InputStream {
    Stdin,
    File(File),
    Pipe(PipeReader),
}

pub enum OutputStream {
    Stdout,
    Stderr,
    File(File),
    Pipe(PipeWriter),
}

pub struct IoStreams {
    pub input: InputStream,
    pub output: OutputStream,
    pub error: OutputStream,
}

impl Default for IoStreams {
    fn default() -> Self {
        Self::new()
    }
}

impl IoStreams {
    pub fn new() -> Self {
        Self {
            input: InputStream::Stdin,
            output: OutputStream::Stdout,
            error: OutputStream::Stderr,
        }
    }

    pub fn try_clone(&self) -> std::io::Result<Self> {
        Ok(IoStreams {
            input: self.input.try_clone()?,
            output: self.output.try_clone()?,
            error: self.error.try_clone()?,
        })
    }
}

impl InputStream {
    pub fn into_stdio(self) -> Stdio {
        match self {
            InputStream::Stdin => Stdio::inherit(),
            InputStream::File(f) => Stdio::from(f),
            InputStream::Pipe(p) => Stdio::from(p),
        }
    }

    pub fn try_clone(&self) -> std::io::Result<Self> {
        match self {
            InputStream::Stdin => Ok(InputStream::Stdin),
            InputStream::File(file) => file.try_clone().map(InputStream::File),
            InputStream::Pipe(reader) => reader.try_clone().map(InputStream::Pipe),
        }
    }
}

impl OutputStream {
    pub fn into_stdio(self) -> Stdio {
        match self {
            OutputStream::Stdout => Stdio::inherit(),
            OutputStream::Stderr => Stdio::inherit(),
            OutputStream::File(f) => Stdio::from(f),
            OutputStream::Pipe(p) => Stdio::from(p),
        }
    }

    pub fn try_clone(&self) -> std::io::Result<Self> {
        match self {
            OutputStream::Stdout => Ok(OutputStream::Stdout),
            OutputStream::Stderr => Ok(OutputStream::Stderr),
            OutputStream::File(file) => file.try_clone().map(OutputStream::File),
            OutputStream::Pipe(writer) => writer.try_clone().map(OutputStream::Pipe),
        }
    }

    pub fn fallback_output_stream(output_stream: &OutputStream) -> OutputStream {
        match output_stream {
            OutputStream::Stdout => OutputStream::Stdout,
            OutputStream::Stderr => OutputStream::Stderr,
            OutputStream::File(f) => OutputStream::File(f.try_clone().unwrap()),
            OutputStream::Pipe(_) => OutputStream::Stderr,
        }
    }
}

impl Write for OutputStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutputStream::Stdout => std::io::stdout().write(buf),
            OutputStream::Stderr => std::io::stderr().write(buf),
            OutputStream::File(f) => f.write(buf),
            OutputStream::Pipe(p) => p.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutputStream::Stdout => std::io::stdout().flush(),
            OutputStream::Stderr => std::io::stderr().flush(),
            OutputStream::File(f) => f.flush(),
            OutputStream::Pipe(p) => p.flush(),
        }
    }
}
