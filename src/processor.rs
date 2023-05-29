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

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_memory::sol_memset,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction::create_account,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    error::BitokuError::{
        ClientMismatch, InvalidAccount, InvalidName, NoAvailableClients, Overflow,
        UnregisteredClient,
    },
    instruction::{BitokuInstructions, Request},
    state::{addel, delel, isel, validate_name, BookKeeper, RequestData},
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = BitokuInstructions::unpack(instruction_data)?;

        match instruction {
            BitokuInstructions::InitBitoku => {
                msg!("Instruction : InitBitoku");
                Self::process_init_bitoku(accounts, program_id)
            }
            BitokuInstructions::RegisterClient => {
                msg!("Instruction : RegisterClient");
                self::Processor::process_register_client(accounts, program_id)
            }

            BitokuInstructions::RemoveClient { client_id } => {
                msg!("Instruction : RemoveClient");
                self::Processor::process_remove_client(accounts, program_id, client_id)
            }

            BitokuInstructions::SendRequest { request, client_id } => {
                msg!("Instruction : SendRequest");
                self::Processor::process_send_request(accounts, program_id, request, client_id)
            }
        }
    }

    fn process_init_bitoku(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_iter = &mut accounts.iter();

        let fee_payer = next_account_info(account_iter)?;
        let bookkeeper = next_account_info(account_iter)?;
        let system_program = next_account_info(account_iter)?;
        let rent_sysvar_account = next_account_info(account_iter)?;

        let rent = Rent::from_account_info(rent_sysvar_account)?;

        let (bookkeeper_key, _bump) =
            Pubkey::find_program_address(&["bookkeeper".as_ref()], program_id);

        if bookkeeper_key != *bookkeeper.key {
            return Err(InvalidAccount.into());
        };

        //creating Bookkeeper account
        let init_bookkeeper = create_account(
            &fee_payer.key,
            &bookkeeper_key,
            rent.minimum_balance(BookKeeper::LEN),
            BookKeeper::LEN as u64,
            &program_id,
        );

        invoke_signed(
            &init_bookkeeper,
            &[
                system_program.clone(),
                fee_payer.clone(),
                bookkeeper.clone(),
            ],
            &[&["bookkeeper".as_ref(), &[_bump]]],
        )?;
        Ok(())
    }

    fn process_register_client(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let fee_payer = next_account_info(accounts_iter)?;
        let bookkeeper = next_account_info(accounts_iter)?;
        let request = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let rent_sys_var = next_account_info(accounts_iter)?;

        let rent = Rent::from_account_info(rent_sys_var)?;

        let (bookkeeper_key, _bump) =
            Pubkey::find_program_address(&["bookkeeper".as_ref()], program_id);

        if bookkeeper_key != *bookkeeper.key {
            return Err(InvalidAccount.into());
        };

        let (request_key, bump) =
            Pubkey::find_program_address(&["request".as_ref(), fee_payer.key.as_ref()], program_id);

        if request_key != *request.key {
            return Err(InvalidAccount.into());
        };

        //creating request account
        let init_request = create_account(
            &fee_payer.key,
            &request_key,
            rent.minimum_balance(RequestData::LEN),
            RequestData::LEN as u64,
            &program_id,
        );

        invoke_signed(
            &init_request,
            &[system_program.clone(), fee_payer.clone(), request.clone()],
            &[&["request".as_ref(), fee_payer.key.as_ref(), &[bump]]],
        )?;

        //getting bookkeeper data from pda
        let mut bookkeeper_data = BookKeeper::unpack_unchecked(&bookkeeper.try_borrow_data()?)?;
        let my_id = bookkeeper_data.next_id;
        addel(&mut bookkeeper_data.status, my_id);
        //getting request_data from pda
        let mut request_data = RequestData::unpack_unchecked(&request.try_borrow_data()?)?;

        request_data.client_id = my_id;
        if bookkeeper_data.next_id == 255 {
            return Err(NoAvailableClients.into());
        }

        bookkeeper_data.next_id += 1;

        BookKeeper::pack(bookkeeper_data, &mut bookkeeper.try_borrow_mut_data()?)?;
        RequestData::pack(request_data, &mut request.try_borrow_mut_data()?)?;

        Ok(())
    }

    fn process_remove_client(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        client_id: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let fee_payer = next_account_info(accounts_iter)?;
        let bookkeeper = next_account_info(accounts_iter)?;
        let request = next_account_info(accounts_iter)?;

        let (bookkeeper_key, _bump) =
            Pubkey::find_program_address(&["bookkeeper".as_ref()], program_id);

        if bookkeeper_key != *bookkeeper.key {
            return Err(InvalidAccount.into());
        };

        let (request_key, _bump) =
            Pubkey::find_program_address(&["request".as_ref(), fee_payer.key.as_ref()], program_id);

        if request_key != *request.key {
            return Err(InvalidAccount.into());
        };

        let mut bookkeeper_data = BookKeeper::unpack_unchecked(&bookkeeper.try_borrow_data()?)?;

        let bool = isel(bookkeeper_data.status, client_id);
        if !bool {
            return Err(UnregisteredClient.into());
        }

        delel(&mut bookkeeper_data.status, client_id);

        BookKeeper::pack(bookkeeper_data, &mut bookkeeper.try_borrow_mut_data()?)?;

        //closing the request PDA account
        let current_lamps = request.lamports();
        let account_data_size = request.data_len();

        //Transferring lamports to the fee_payer account
        **request.lamports.borrow_mut() = 0;
        **fee_payer.lamports.borrow_mut() = fee_payer
            .lamports()
            .checked_add(current_lamps)
            .ok_or(Overflow)?;

        //zeroing the stored data in the account
        sol_memset(&mut *request.try_borrow_mut_data()?, 0, account_data_size);

        Ok(())
    }

    fn process_send_request(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        request: Request,
        client_id: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let fee_payer = next_account_info(accounts_iter)?;
        let req = next_account_info(accounts_iter)?;

        let (request_key, _bump) =
            Pubkey::find_program_address(&["request".as_ref(), fee_payer.key.as_ref()], program_id);

        if request_key != *req.key {
            return Err(InvalidAccount.into());
        };

        let mut request_data = RequestData::unpack_unchecked(&req.try_borrow_data()?)?;

        //Validating the name of the request
        let s = request.name();
        let name = String::from_utf8(s.to_vec()).unwrap();
        if validate_name(&name.as_bytes()) == false {
            return Err(InvalidName.into());
        }

        if request_data.client_id != client_id {
            msg!("{} != {}", request_data.client_id, client_id);
            return Err(ClientMismatch.into());
        }

        request_data.requester = *fee_payer.key;
        request_data.request = request;

        RequestData::pack(request_data, &mut req.try_borrow_mut_data()?)?;

        Ok(())
    }
}
