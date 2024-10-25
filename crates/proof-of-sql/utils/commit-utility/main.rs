//! Utility to deserialize and print a commitment from a file or stdin.
use clap::Parser;
use curve25519_dalek::ristretto::RistrettoPoint;
use proof_of_sql::{
    base::commitment::TableCommitment,
    proof_primitive::dory::{DoryCommitment, DynamicDoryCommitment},
};
use snafu::Snafu;
use std::{
    fs::File,
    io::{self, Read, Write},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file (defaults to stdin)
    #[arg(short, long)]
    input: Option<String>,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Commitment scheme (e.g. `ipa`, `dynamic_dory`, `dory`)
    #[arg(long)]
    scheme: String,
}

#[derive(Debug, Snafu)]
enum CommitUtilityError {
    #[snafu(display("Failed to open input file '{}'", filename))]
    OpenInputFile { filename: String },

    #[snafu(display("Failed to read from input file '{}'", filename))]
    ReadInputFile { filename: String },

    #[snafu(display("Failed to read from stdin"))]
    ReadStdin,

    #[snafu(display("Failed to create output file '{}'", filename))]
    CreateOutputFile { filename: String },

    #[snafu(display("Failed to write to output file '{}'", filename))]
    WriteOutputFile { filename: String },

    #[snafu(display("Failed to write to stdout"))]
    WriteStdout,

    #[snafu(display("Failed to deserialize commitment"))]
    DeserializationError,

    #[snafu(display("Unknown scheme: '{}'", scheme))]
    UnknownScheme { scheme: String },
}

type CommitUtilityResult<T, E = CommitUtilityError> = std::result::Result<T, E>;

fn main() -> CommitUtilityResult<()> {
    let cli = Cli::parse();

    // Read input data
    let input_data = match &cli.input {
        Some(input_file) => {
            let mut file =
                File::open(input_file).map_err(|_| CommitUtilityError::OpenInputFile {
                    filename: input_file.clone(),
                })?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|_| CommitUtilityError::ReadInputFile {
                    filename: input_file.clone(),
                })?;
            buffer
        }
        None => {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|_| CommitUtilityError::ReadStdin)?;
            buffer
        }
    };

    // Deserialize commitment based on the scheme
    let human_readable = match cli.scheme.to_lowercase().as_str() {
        "dynamic_dory" | "dynamic-dory" => {
            let commitment: TableCommitment<DynamicDoryCommitment> =
                postcard::from_bytes(&input_data)
                    .map_err(|_| CommitUtilityError::DeserializationError)?;
            format!("{commitment:#?}")
        }
        "dory" => {
            let commitment: TableCommitment<DoryCommitment> = postcard::from_bytes(&input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            format!("{commitment:#?}")
        }
        "ipa" | "innerproductargument" => {
            let commitment: TableCommitment<RistrettoPoint> = postcard::from_bytes(&input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            format!("{commitment:#?}")
        }
        _ => {
            return Err(CommitUtilityError::UnknownScheme {
                scheme: cli.scheme.clone(),
            });
        }
    };

    // Write output data
    match &cli.output {
        Some(output_file) => {
            let mut file =
                File::create(output_file).map_err(|_| CommitUtilityError::CreateOutputFile {
                    filename: output_file.clone(),
                })?;
            file.write_all(human_readable.as_bytes()).map_err(|_| {
                CommitUtilityError::WriteOutputFile {
                    filename: output_file.clone(),
                }
            })?;
        }
        None => {
            io::stdout()
                .write_all(human_readable.as_bytes())
                .map_err(|_| CommitUtilityError::WriteStdout)?;
        }
    }

    Ok(())
}