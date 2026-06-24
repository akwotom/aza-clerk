/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for creating necessary stored procedures.
*/

use crate::logic::db::DbHandle;
use crate::logic::sql_utils;

const BEGIN_TRANSFER: &str = "
    CREATE PROCEDURE IF NOT EXISTS begin_transfer (
        IN txn_id VARCHAR(64),
        IN src_user_id TEXT, 
        IN bene_user_id TEXT, 
        IN amt_value INT, 
        IN amt_currency TEXT
    )
    BEGIN
        -- First, let's make sure this transfer has not already happened.

        -- This variable contains the sending user's account balance, computed by another stored procedure
        DECLARE src_balance INT DEFAULT 0;
        DECLARE src_account_id TEXT;
        DECLARE bene_account_id TEXT;
        DECLARE existing_txn TEXT;


        CALL compute_account_balance (src_user_id, amt_currency, src_balance, src_account_id);


        -- Now, if the balance is too low, let's throw an error
        IF src_balance < amt_value THEN
            SIGNAL SQLSTATE '45008'
                SET MESSAGE_TEXT = 'User balance is insufficient';
        END IF;

    
        -- We're obtaining a lock in the ledger for any debiting to that account id
        -- We do this before even computing the user's account balance, in order that the user 
        -- doesn't spend money he doesn't have.
        SELECT * FROM ledger 
            WHERE 
                account_id = src_account_id
        FOR UPDATE;

        -- Now, let's create the transfer, then debit the sending user.



        START TRANSACTION;
            -- Create the transaction first

            -- But wait! What if the transaction already exist?

            SELECT 
                id INTO existing_txn
            FROM
                transactions
            WHERE
                id = txn_id
            LIMIT 1;

            IF existing_txn IS NOT NULL AND existing_txn <> '' 
            THEN
                SIGNAL SQLSTATE '45009'
                    SET MESSAGE_TEXT = 'Transaction already exists';
            END IF;

            CALL get_or_create_account (src_user_id, amt_currency, bene_account_id);

            -- And if the transaction doesn't exist, let's create it

            INSERT INTO transactions (
                id,
                action,
                amount_value,
                amount_currency,
                transfer_src_user_id,
                transfer_src_account_id,
                transfer_bene_user_id,
                transfer_bene_account_id,
                status
            ) VALUES (
                txn_id,
                'transfer',
                amt_value,
                amt_currency,
                src_user_id,
                src_account_id,
                bene_user_id,
                bene_account_id,
                'pending'
            );
            

            INSERT INTO ledger (
                account_id,
                user_id,
                amount,
                transaction_id
            )
            VALUES (
                src_account_id,
                src_user_id,
                amt_value * -1, -- We're debiting the source account
                txn_id
            );

        COMMIT;

    END;
";

const COMPUTE_BALANCE: &str = "
    CREATE PROCEDURE IF NOT EXISTS compute_account_balance (
        IN in_user_id TEXT,
        IN in_currency TEXT,
        OUT out_balance INT,
        OUT out_account_id TEXT
    )
    BEGIN
        -- To compute the user's balance, by summing all amounts for the given account id.

        SELECT
            (
                CAST( COALESCE( (SELECT SUM(amount) FROM ledger WHERE account_id = accounts.id AND user_id = accounts.user_id), 0) AS SIGNED)
            ),
            id
        INTO
            out_balance,
            out_account_id
        FROM 
            accounts
        WHERE
            currency = in_currency
        AND
            user_id = in_user_id
        LIMIT 1;
        
        IF out_balance IS NULL THEN
            SET out_balance = 0;
        END IF;
    END;
";

const COMPLETE_TRANSFER: &str = "
    CREATE PROCEDURE IF NOT EXISTS complete_transfer (
        IN transaction_id TEXT
    )
    complete_transfer: BEGIN

        DECLARE
            src_account_id,
            bene_account_id,
            bene_user_id,
            existing_state TEXT;
        DECLARE 
            txn_amount INT;


        SELECT 
                status,
                transfer_src_account_id,
                transfer_bene_account_id,
                transfer_bene_user_id,
                amount_value
            INTO 
                existing_state,
                src_account_id, 
                bene_account_id,
                bene_user_id,
                txn_amount
        FROM
            transactions 
        WHERE
            id = transaction_id
        LIMIT 1
        FOR UPDATE;

        IF src_account_id IS NULL THEN
            SIGNAL SQLSTATE '45011' 
                SET MESSAGE_TEXT = 'The transfer being completed was not found.';
        END IF;

        IF existing_state = 'successful'
        THEN
            LEAVE complete_transfer;
        END IF;

        IF existing_state <> 'pending'
        THEN
            SIGNAL SQLSTATE '45010'
                SET MESSAGE_TEXT = 'The transfer is not in a state that can be completed.';
        END IF;


        START TRANSACTION;
            INSERT INTO ledger (
                account_id,
                user_id,
                amount,
                transaction_id
            )
            VALUES (
                bene_account_id,
                bene_user_id,
                txn_amount,
                transaction_id
            );

            UPDATE 
                transactions
            SET
                status = 'successful';
        COMMIT;
        
    END
";

// NOTE: A trade-off here involves early locking.
// It might cause more locking, but it prevents us from making multiple queries.
// In the end, it saves us more time.
const FAIL_TRANSFER: &str = "
    CREATE PROCEDURE IF NOT EXISTS reverse_transfer  (
        IN txn_id TEXT
    ) 
    reverse_transfer: BEGIN
        DECLARE 
            src_account_id,
            src_user_id,
            txn_state TEXT;
        SELECT
            transfer_src_account_id,
            transfer_src_user_id,
            status 
        INTO 
            src_account_id,
            src_user_id,
            txn_state
        FROM transactions
        WHERE
            id = txn_id
        FOR UPDATE;

        IF status = 'reversed'
        THEN
            LEAVE reverse_transfer;
        END IF;

        IF STATUS <> 'pending'
        THEN
            SIGNAL SQLSTATE '45011'
                SET MESSAGE_TEXT = 'The transfer cannot be reversed in this state';
        END IF;

        START TRANSACTION;
            UPDATE transactions
            SET 
                status = 'reversed'
            WHERE
                id = txn_id;

            INSERT INTO ledger (
                account_id,
                user_id,
                amount,
                transaction_id
            )
            VALUES (
                src_account_id,
                src_user_id,
                txn_amount * -1,
                txn_id
            );
        COMMIT;

    END
";

const ENSURE_ACCOUNT: &str = "
    CREATE PROCEDURE IF NOT EXISTS get_or_create_account (
        IN in_user_id TEXT,
        IN in_currency TEXT,
        OUT out_account_id TEXT
    )
    get_if_exists: BEGIN
        SELECT 
            id INTO out_account_id 
        FROM 
            accounts 
        WHERE 
                user_id = in_user_id 
            AND 
                currency = in_currency
        LIMIT 1;

        IF 
            out_account_id IS NOT NULL 
        AND 
            out_account_id <> ''
        THEN
            LEAVE get_if_exists;

        END IF;
        
        SET out_account_id = UUID();

        INSERT INTO accounts (id, user_id, currency) VALUES (out_account_id, in_user_id, in_currency);

    END
";

const CREDIT_USER: &str = "
    CREATE PROCEDURE IF NOT EXISTS credit_user (
        IN txn_id TEXT,
        IN user_id TEXT,
        IN amt_value INT,
        IN amt_currency TEXT
    )
    credit_user: BEGIN
        DECLARE 
            existing_txn,
            user_account_id,
            system_account_id
        TEXT;

        SELECT 
            id
        INTO
            existing_txn
        FROM
            transactions
        WHERE
            id = txn_id
        LIMIT 1
        FOR UPDATE;

        IF existing_txn IS NOT NULL AND existing_txn <> ''
        THEN
            SIGNAL SQLSTATE '45009'
                SET MESSAGE_TEXT = 'Transaction already exists';
        END IF;



        
        START TRANSACTION;

            CALL get_or_create_account (user_id, amt_currency, user_account_id);
            CALL get_or_create_account ('system_user_id', amt_currency, system_account_id);



            INSERT INTO transactions (
                id,
                action,
                amount_value,
                amount_currency,
                top_up_bene_user_id,
                top_up_bene_account_id,
                status
            ) VALUES (
                txn_id,
                'top_up',
                amt_value,
                amt_currency,
                user_id,
                user_account_id,
                'successful'
            );

            -- Now, Let's credit the user on the ledger

            INSERT INTO ledger (
                account_id,
                user_id,
                amount,
                transaction_id
            )
            VALUES (
                user_account_id,
                user_id,
                amt_value,
                txn_id
            );

            -- In accounting, when crediting a user of a double-ledger system, we need to debit the system account
            -- The reasoning, is that we owe the user his money. We have a liability to take care of.
            INSERT INTO ledger (
                account_id,
                user_id,
                amount,
                transaction_id
            )
            VALUES (
                system_account_id,
                'system_user_id',
                amt_value * -1,
                txn_id
            );
        COMMIT;
            
    END
";

/// This method creates the SQL stored procedures in DB.
pub async fn init_sql_procedures(db: &DbHandle) -> Result<(), sqlx::Error> {
    sql_utils::run_multiple_sql(
        &[
            BEGIN_TRANSFER,
            COMPLETE_TRANSFER,
            FAIL_TRANSFER,
            COMPUTE_BALANCE,
            ENSURE_ACCOUNT,
            CREDIT_USER,
        ],
        db,
    )
    .await
}
