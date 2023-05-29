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

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
    pubkey::Pubkey,
};

use crate::instruction::{unpack_request, Request};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct BookKeeper {
    pub status: [u8; 32],
    pub next_id: u8,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RequestData {
    pub client_id: u8,
    pub requester: Pubkey,
    pub request: Request,
}

impl Sealed for BookKeeper {}

impl Pack for BookKeeper {
    const LEN: usize = 33;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < 33 {
            return Err(ProgramError::InvalidAccountData);
        }

        let status: [u8; 32] = src[..32].try_into().unwrap();
        let next_id = u8::from_le_bytes(src[32..33].try_into().unwrap())
            .try_into()
            .unwrap();

        Ok(Self { status, next_id })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let status = self.status;
        let next_id = self.next_id.to_le_bytes();

        for i in 0..32 {
            dst[i] = status[i];
        }

        for i in 32..33 {
            dst[i] = next_id[0];
        }
    }
}

impl Sealed for RequestData {}

impl Pack for RequestData {
    const LEN: usize = 1 + 32 + 1 + 128 + 1 + 512;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < RequestData::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let client_id = u8::from_le_bytes(src[..1].try_into().unwrap())
            .try_into()
            .unwrap();
        let requester = Pubkey::new(&src[1..33]);
        let request_bytes = &src[33..];

        let request = unpack_request(request_bytes)?;

        Ok(Self {
            client_id,
            requester,
            request,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let client_id = self.client_id;
        let requester = self.requester.to_bytes();

        dst[0] = client_id;

        for i in 1..33 {
            dst[i] = requester[i - 1]
        }

        match &self.request {
            Request::CreateBucket { name } => {
                dst[33] = 0;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
            }
            Request::CreateFile { name, data } => {
                dst[33] = 1;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
                dst[162] = 0;
                for i in 163..675 {
                    dst[i] = data[i - 163]
                }
            }
            Request::WriteFile {
                name,
                file_id,
                data,
            } => {
                dst[33] = 2;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
                dst[162] = *file_id;
                for i in 163..675 {
                    dst[i] = data[i - 163]
                }
            }
            Request::CloseFile { name, file_id } => {
                dst[33] = 3;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
                dst[162] = *file_id;
            }
            Request::DeleteFile { name, file_id } => {
                dst[33] = 4;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
                dst[162] = *file_id;
            }
            Request::SetPosition {
                name,
                file_id,
                position,
            } => {
                dst[33] = 5;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
                dst[162] = *file_id;

                let position_bytes = position.to_le_bytes();

                for i in 163..171 {
                    dst[i] = position_bytes[i - 163]
                }
            }
            Request::OpenFile { name, file_id } => {
                dst[33] = 6;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
                dst[162] = *file_id;
            }
            Request::ReadFile { name, file_id } => {
                dst[33] = 7;
                for i in 34..162 {
                    dst[i] = name[i - 34];
                }
                dst[162] = *file_id;
            }
        }
    }
}

pub fn addel(src: &mut [u8; 32], element: u8) {
    let byte_index = element / 8;
    let bit_offset = element % 8;
    src[byte_index as usize] |= 1 << bit_offset;
}

pub fn isel(src: [u8; 32], element: u8) -> bool {
    let byte_index = element / 8;
    let bit_offset = element % 8;
    let value = (src[byte_index as usize] >> bit_offset) & 1;

    if value == 1 {
        return true;
    }
    false
}

pub fn delel(src: &mut [u8; 32], element: u8) {
    let byte_index = element / 8;
    let bit_offset = element % 8;
    src[byte_index as usize] &= !(1 << bit_offset);
}

pub fn validate_name(name: &[u8]) -> bool {
    if name.len() > 128 as usize {
        return false;
    }

    let non_zero_bytes: Vec<u8> = name.iter().take_while(|&b| *b != 0).copied().collect();

    for b in non_zero_bytes {
        if !(b >= b'a' && b <= b'z')
            && !(b >= b'A' && b <= b'Z')
            && !(b >= b'0' && b <= b'9')
            && !(b == b'.' || b == b'/' || b == b'_' || b == b'+' || b == b'-')
        {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod test {

    use super::*;
    use super::{Request, RequestData};
    #[test]
    fn test_pack() {
        let mut name: [u8; 128] = [0; 128];
        let bytes = "test".as_bytes();
        name[..bytes.len()].copy_from_slice(bytes);

        let requester = Pubkey::new_unique();
        print!("name {:?}", String::from_utf8(name.to_vec()).unwrap());
        let src = RequestData {
            client_id: 85,
            requester: requester,
            request: Request::CloseFile { name, file_id: 69 },
        };
        let mut dst = [0u8; 163];
        println!("{:?}", src);

        let res = RequestData::pack(src, &mut dst);
        print!("packed {:?}", res.unwrap());
    }
    #[test]
    fn test_name_validation() {
        let name = "test".as_bytes();

        let bool = validate_name(&name);

        assert!(bool);
    }
}
