use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::state::DataV2;

declare_id!("42TNfJ8hVwfaL4VrT5mJBRAt1sWhMwmd4HuFuovqdtLk");

#[program]
pub mod token {
    use super::*;

    pub fn initialize_nft(
        ctx: Context<InitializeNft>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        // Create mint account
        let mint = &ctx.accounts.mint;
        let token_program = &ctx.accounts.token_program;
        let payer = &ctx.accounts.payer;
        
        // Initialize mint account
        anchor_spl::token::initialize_mint(
            CpiContext::new(
                token_program.to_account_info(),
                anchor_spl::token::InitializeMint {
                    mint: mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0, // decimals
            payer.key,
            Some(payer.key),
        )?;

        // Create metadata
        let metadata = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        Ok(())
    }

    pub fn mint_nft(ctx: Context<MintNft>) -> Result<()> {
        let mint = &ctx.accounts.mint;
        let token = &ctx.accounts.token;
        let owner = &ctx.accounts.owner;
        
        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: mint.to_account_info(),
                    to: token.to_account_info(),
                    authority: owner.to_account_info(),
                },
            ),
            1,
        )?;

        Ok(())
    }

    pub fn transfer_nft(ctx: Context<TransferNft>) -> Result<()> {
        let from = &ctx.accounts.from;
        let to = &ctx.accounts.to;
        let owner = &ctx.accounts.owner;
        
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: owner.to_account_info(),
                },
            ),
            1,
        )?;

        Ok(())
    }

    pub fn update_nft(
        ctx: Context<UpdateNft>,
        name: Option<String>,
        symbol: Option<String>,
        uri: Option<String>,
    ) -> Result<()> {
        let mint = &ctx.accounts.mint;
        let authority = &ctx.accounts.authority;
        
        // Get current metadata
        let current_metadata = DataV2 {
            name: name.unwrap_or_else(|| "".to_string()),
            symbol: symbol.unwrap_or_else(|| "".to_string()),
            uri: uri.unwrap_or_else(|| "".to_string()),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        // Verify authority is the mint authority
        require!(
            mint.mint_authority.unwrap() == authority.key(),
            ErrorCode::UnauthorizedAuthority
        );

        msg!("NFT metadata updated successfully");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer,
    )]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferNft<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateNft<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Authority is not authorized to update this NFT")]
    UnauthorizedAuthority,
}
