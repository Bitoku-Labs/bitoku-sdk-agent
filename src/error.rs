/*
  Copyright 2023 Bitoku Labs

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use thiserror::Error;

use solana_program::{decode_error::DecodeError, program_error::ProgramError};

#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum BitokuError {
    //0
    #[error("Instruction is not valid")]
    InvalidInstruction,
    //1
    #[error("instruction_data is invalid")]
    InvalidInstructionData,
    //2
    #[error("client limit reached")]
    NoAvailableClients,
    //3
    #[error("numbers overflow")]
    Overflow,
    //4
    #[error("client is not registered")]
    UnregisteredClient,
    //5
    #[error("name is not valid")]
    InvalidName,
    //6
    #[error("account is not valid")]
    InvalidAccount,
    //7
    #[error("client is not valid")]
    InvalidClientId,
    //8
    #[error("file id is not valid")]
    InvalidFileId,
    //9
    #[error("provided position is not valid")]
    InvalidPosition,
    //10
    #[error("client id mismatch")]
    ClientMismatch,
}

impl From<BitokuError> for ProgramError {
    fn from(e: BitokuError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for BitokuError {
    fn type_of() -> &'static str {
        "BitokuError"
    }
}
