use crate::engine::exit::ExitCode;
use crate::io::redirection::{RedirectionMode, initialise_writer_file};
use crate::parser::command_node::{CommandNode, Redirection};
use crate::parser::word::words_to_string;
use crate::shell::variables::Variables;
use reedline::ExternalPrinter;
use std::fs::File;
use std::io::{BufRead, PipeReader, PipeWriter, Write};
use std::iter::Peekable;
use std::process::Stdio;
use std::thread::JoinHandle;
use std::vec::IntoIter;

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

    pub fn apply_redirection(&mut self, redirection: Redirection, variables: &Variables) {
        if redirection.mode != RedirectionMode::Nothing && !redirection.path.is_empty() {
            let file = initialise_writer_file(
                &redirection.mode,
                &words_to_string(redirection.path, variables),
            );

            match redirection.mode {
                RedirectionMode::Out | RedirectionMode::OutAppend => {
                    self.output = OutputStream::File(file);
                }
                RedirectionMode::Error | RedirectionMode::ErrorAppend => {
                    self.error = OutputStream::File(file);
                }
                RedirectionMode::Nothing => {
                    self.output = OutputStream::Stdout;
                    self.error = OutputStream::Stderr;
                }
            }
        }
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

pub fn create_pipe() -> Result<(PipeReader, PipeWriter), ExitCode> {
    match std::io::pipe() {
        Ok(pipes) => Ok(pipes),
        Err(err) => {
            eprintln!("shell: pipe creation failed: {err}");

            Err(ExitCode::FAILURE)
        }
    }
}

pub fn pipe_io_streams(
    current_input: InputStream,
    parent_output: OutputStream,
    parent_error: OutputStream,
    iter: &mut Peekable<IntoIter<CommandNode>>,
) -> Result<(IoStreams, InputStream), ExitCode> {
    let (next_input, current_output) = if iter.peek().is_some() {
        let (read_end, write_end) = create_pipe()?;
        (InputStream::Pipe(read_end), OutputStream::Pipe(write_end))
    } else {
        (InputStream::Stdin, parent_output)
    };

    Ok((
        IoStreams {
            input: current_input,
            output: current_output,
            error: parent_error,
        },
        next_input,
    ))
}

pub fn background_io_streams(
    printer: &ExternalPrinter<String>,
) -> Result<(IoStreams, JoinHandle<()>, JoinHandle<()>), ExitCode> {
    let (read_out, write_out) = create_pipe()?;
    let (read_err, write_err) = create_pipe()?;

    let out_reader_printer = spawn_background_reader_printer(printer.clone(), read_out);
    let err_reader_printer = spawn_background_reader_printer(printer.clone(), read_err);

    let bg_io_streams = IoStreams {
        input: InputStream::Stdin,
        output: OutputStream::Pipe(write_out),
        error: OutputStream::Pipe(write_err),
    };

    Ok((bg_io_streams, out_reader_printer, err_reader_printer))
}

fn spawn_background_reader_printer(
    printer: ExternalPrinter<String>,
    pipe_reader: PipeReader,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(pipe_reader);
        for line in reader.lines().map_while(Result::ok) {
            let _ = printer.print(line);
        }
    })
}
