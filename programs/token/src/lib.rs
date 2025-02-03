use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self as spl_token, Token, Mint, TokenAccount},
};
use mpl_token_metadata::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Creator, DataV2},
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
        // Create metadata account
        let seeds = &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            ctx.accounts.mint.key().as_ref(),
        ];
        let (metadata_account, _) = Pubkey::find_program_address(seeds, &mpl_token_metadata::ID);

        let creators = vec![Creator {
            address: ctx.accounts.payer.key(),
            verified: true,
            share: 100,
        }];

        let data_v2 = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(creators),
            collection: None,
            uses: None,
        };

        let ix = CreateMetadataAccountV3 {
            metadata: metadata_account,
            mint: ctx.accounts.mint.key(),
            mint_authority: ctx.accounts.payer.key(),
            payer: ctx.accounts.payer.key(),
            update_authority: ctx.accounts.payer.key(),
            system_program: ctx.accounts.system_program.key(),
            rent: ctx.accounts.rent.key(),
        }
        .instruction(CreateMetadataAccountV3InstructionArgs {
            data: data_v2,
            is_mutable: true,
            collection_details: None,
        });

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        // Initialize mint
        spl_token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                spl_token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.payer.key(),
            Some(&ctx.accounts.payer.key()),
        )?;

        Ok(())
    }

    pub fn mint_nft(ctx: Context<MintNft>) -> Result<()> {
        spl_token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                spl_token::MintTo {
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
        spl_token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                spl_token::Transfer {
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
#[instruction(name: String, symbol: String, uri: String)]
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
    
    /// CHECK: Handled by Metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
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
