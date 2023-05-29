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

use crate::error::BitokuError::{
    InvalidClientId, InvalidFileId, InvalidInstruction, InvalidInstructionData, InvalidPosition,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem::size_of;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum Request {
    CreateBucket {
        name: [u8; 128],
    },
    CreateFile {
        name: [u8; 128],
        data: [u8; 512],
    },
    WriteFile {
        name: [u8; 128],
        file_id: u8,
        data: [u8; 512],
    },
    CloseFile {
        name: [u8; 128],
        file_id: u8,
    },
    DeleteFile {
        name: [u8; 128],
        file_id: u8,
    },
    SetPosition {
        name: [u8; 128],
        file_id: u8,
        position: u64,
    },
    OpenFile {
        name: [u8; 128],
        file_id: u8,
    },
    ReadFile {
        name: [u8; 128],
        file_id: u8,
    },
}

#[derive(BorshSerialize,BorshDeserialize,Debug, Clone)]
#[rustfmt::skip]
pub enum BitokuInstructions {
    ///0. `[signer]` fee_payer account
    /// 1. `[writable]` bookkeeper PDA account
    /// 2.`[]` system_program account
    /// 3.`[]` sys_var program
    InitBitoku,
    ///0. `[signer]` fee_payer account
    /// 1. `[writable]` bookkeeper PDA account
    /// 2. `[]` request Pda account
    /// 3.`[]` system_program account
    ///  4.`[]` sys_var program
    RegisterClient,

    ///0. `[signer]` fee_payer account
    /// 1. `[writable]` bookkeeper PDA account
    /// 2. `[writable]` request Pda account
    RemoveClient{client_id:u8},

    ///0. `[signer]` fee_payer account
    /// 2. `[writable]` request Pda account
    SendRequest{client_id : u8,request : Request}
}

impl BitokuInstructions {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstructionData)?;

        let (_, request): (&u8, &[u8]);
        if *tag == 3 {
            (_, request) = rest.split_first().ok_or(InvalidInstructionData)?;
        } else {
            request = &[0u8];
        }
        Ok(match tag {
            0 => Self::InitBitoku {},
            1 => Self::RegisterClient {},
            2 => Self::RemoveClient {
                client_id: unpack_client_id(rest)?,
            },
            3 => Self::SendRequest {
                client_id: unpack_client_id(rest)?,
                request: unpack_request(request)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::InitBitoku => {
                buf.push(0);
            }
            Self::RegisterClient => {
                buf.push(1);
            }

            Self::RemoveClient { client_id } => {
                buf.push(2);
                buf.extend_from_slice(&client_id.to_le_bytes());
            }
            Self::SendRequest { request, client_id } => {
                buf.push(3);
                buf.extend_from_slice(&client_id.to_le_bytes());
                match request {
                    Request::CreateBucket { name } => {
                        buf.push(0);
                        buf.extend_from_slice(name);
                    }
                    Request::CreateFile { name, data } => {
                        buf.push(1);
                        buf.extend_from_slice(name);
                        buf.extend_from_slice(data);
                    }
                    Request::WriteFile {
                        name,
                        file_id,
                        data,
                    } => {
                        buf.push(2);
                        buf.extend_from_slice(name);
                        buf.extend_from_slice(&file_id.to_le_bytes());
                        buf.extend_from_slice(data);
                    }
                    Request::CloseFile { name, file_id } => {
                        buf.push(3);
                        buf.extend_from_slice(name);
                        buf.extend_from_slice(&file_id.to_le_bytes());
                    }
                    Request::DeleteFile { name, file_id } => {
                        buf.push(4);
                        buf.extend_from_slice(name);
                        buf.extend_from_slice(&file_id.to_le_bytes());
                    }
                    Request::SetPosition {
                        name,
                        file_id,
                        position,
                    } => {
                        buf.push(5);
                        buf.extend_from_slice(name);
                        buf.extend_from_slice(&file_id.to_le_bytes());
                        buf.extend_from_slice(&position.to_le_bytes())
                    }
                    Request::OpenFile { name, file_id } => {
                        buf.push(6);
                        buf.extend_from_slice(name);
                        buf.extend_from_slice(&file_id.to_le_bytes());
                    }
                    Request::ReadFile { name, file_id } => {
                        buf.push(7);
                        buf.extend_from_slice(name);
                        buf.extend_from_slice(&file_id.to_le_bytes());
                    }
                }
            }
        };
        buf
    }
}

pub fn unpack_request(input: &[u8]) -> Result<Request, ProgramError> {
    let (req, data) = input.split_first().ok_or(InvalidInstructionData)?;

    let name = unpack_name(data)?;

    Ok(match req {
        0 => self::Request::CreateBucket { name },
        1 => self::Request::CreateFile {
            name,
            data: unpack_data(data)?,
        },
        2 => self::Request::WriteFile {
            name,
            file_id: unpack_file_id(data)?,
            data: unpack_data(data)?,
        },
        3 => self::Request::CloseFile {
            name,
            file_id: unpack_file_id(data)?,
        },
        4 => self::Request::DeleteFile {
            name,
            file_id: unpack_file_id(data)?,
        },
        5 => self::Request::SetPosition {
            name,
            file_id: unpack_file_id(data)?,
            position: unpack_position(data)?,
        },
        6 => self::Request::OpenFile {
            name,
            file_id: unpack_file_id(data)?,
        },
        7 => self::Request::ReadFile {
            name,
            file_id: unpack_file_id(data)?,
        },
        _ => return Err(InvalidInstruction.into()),
    })
}

fn unpack_client_id(input: &[u8]) -> Result<u8, ProgramError> {
    let id = input
        .get(..1)
        .and_then(|slice| slice.try_into().ok())
        .map(u8::from_be_bytes)
        .ok_or(InvalidClientId)?;
    Ok(id)
}

fn unpack_name(input: &[u8]) -> Result<[u8; 128], ProgramError> {
    let name = input
        .get(..128)
        .and_then(|slice| slice.try_into().ok())
        .unwrap();
    Ok(name)
}

fn unpack_file_id(input: &[u8]) -> Result<u8, ProgramError> {
    let id = input
        .get(128..129)
        .and_then(|slice| slice.try_into().ok())
        .map(u8::from_be_bytes)
        .ok_or(InvalidFileId)?;
    Ok(id)
}

fn unpack_data(input: &[u8]) -> Result<[u8; 512], ProgramError> {
    let data = input.get(129..).unwrap();
    let mut padded_data = [0u8; 512];
    padded_data[..data.len()].copy_from_slice(data);
    Ok(padded_data)
}

fn unpack_position(input: &[u8]) -> Result<u64, ProgramError> {
    let position = input
        .get(129..137)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(InvalidPosition)?;
    Ok(position)
}

impl Request {
    pub fn name(&self) -> [u8; 128] {
        match self {
            Request::CreateBucket { name } => *name,
            Request::CreateFile { name, .. } => *name,
            Request::WriteFile { name, .. } => *name,
            Request::DeleteFile { name, .. } => *name,
            Request::CloseFile { name, .. } => *name,
            Request::SetPosition { name, .. } => *name,
            Request::OpenFile { name, .. } => *name,
            Request::ReadFile { name, .. } => *name,
        }
    }
}

pub fn register_client(
    fee_payer: Pubkey,
    bookkeeper: Pubkey,
    request: Pubkey,
    system_program: Pubkey,
    rent_sys_var: Pubkey,
    bitoku_agnet_program: Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = BitokuInstructions::RegisterClient {}.pack();

    let accounts = vec![
        AccountMeta::new(fee_payer, true),
        AccountMeta::new(bookkeeper, false),
        AccountMeta::new(request, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(rent_sys_var, false),
    ];

    Ok(Instruction {
        program_id: bitoku_agnet_program,
        accounts,
        data,
    })
}

pub fn remove_client(
    fee_payer: Pubkey,
    bookkeeper: Pubkey,
    request: Pubkey,
    bitoku_agnet_program: Pubkey,
    client_id: u8,
) -> Result<Instruction, ProgramError> {
    let data = BitokuInstructions::RemoveClient { client_id }.pack();

    let accounts = vec![
        AccountMeta::new(fee_payer, true),
        AccountMeta::new(bookkeeper, false),
        AccountMeta::new(request, false),
    ];

    Ok(Instruction {
        program_id: bitoku_agnet_program,
        accounts,
        data,
    })
}

pub fn send_request(
    fee_payer: Pubkey,
    request: Pubkey,
    bitoku_agnet_program: Pubkey,
    client_id: u8,
    req: Request,
) -> Result<Instruction, ProgramError> {
    let data = BitokuInstructions::SendRequest {
        client_id,
        request: req,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(fee_payer, true),
        AccountMeta::new(request, false),
    ];

    Ok(Instruction {
        program_id: bitoku_agnet_program,
        accounts,
        data,
    })
}
