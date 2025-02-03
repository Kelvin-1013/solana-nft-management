use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, Mint, TokenAccount},
};

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
        let nft_metadata = &mut ctx.accounts.nft_metadata;
        let mint = &ctx.accounts.mint;
        let payer = &ctx.accounts.payer;

        // Initialize metadata
        nft_metadata.name = name;
        nft_metadata.symbol = symbol;
        nft_metadata.uri = uri;
        nft_metadata.mint = mint.key();
        nft_metadata.authority = payer.key();
        
        // Initialize mint account
        anchor_spl::token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::InitializeMint {
                    mint: mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0, // decimals
            payer.key,
            Some(payer.key),
        )?;

        Ok(())
    }

    pub fn mint_nft(ctx: Context<MintNft>) -> Result<()> {
        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            1,
        )?;

        Ok(())
    }

    pub fn transfer_nft(ctx: Context<TransferNft>) -> Result<()> {
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
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
        let metadata = &mut ctx.accounts.nft_metadata;
        let authority = &ctx.accounts.authority;

        require!(
            metadata.authority == authority.key(),
            ErrorCode::UnauthorizedAuthority
        );

        if let Some(name) = name {
            metadata.name = name;
        }

        if let Some(symbol) = symbol {
            metadata.symbol = symbol;
        }

        if let Some(uri) = uri {
            metadata.uri = uri;
        }

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
        space = 8 + 82,  // Discriminator + min space for Mint
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        space = 8 + NFTMetadata::LEN,
        seeds = [b"metadata", mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NFTMetadata>,
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
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"metadata", mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NFTMetadata>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(Default)]
pub struct NFTMetadata {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

impl NFTMetadata {
    pub const LEN: usize = 32 + // mint
        32 + // authority
        4 + 32 + // name string
        4 + 10 + // symbol string
        4 + 200; // uri string
}

#[error_code]
pub enum ErrorCode {
    #[msg("Authority is not authorized to update this NFT")]
    UnauthorizedAuthority,
}
