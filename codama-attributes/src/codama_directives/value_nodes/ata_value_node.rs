use crate::utils::SetOnce;
use codama_nodes::{
    AccountValueNode, PdaNode, PdaSeedValueNode, PdaValueNode, PublicKeyTypeNode,
    VariablePdaSeedNode,
};
use codama_syn_helpers::{extensions::*, Meta};

const ATA_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// Parses `ata("owner_account", "mint_account", "token_program_account")` or
/// `ata(owner = "...", mint = "...", token_program = "...")` into a `PdaValueNode`
/// with an inline `PdaNode` for the Associated Token Account program.
pub fn pda_value_node_from_ata(meta: &Meta) -> syn::Result<PdaValueNode> {
    let pl = meta.assert_directive("ata")?.as_path_list()?;
    let mut owner = SetOnce::<String>::new("owner");
    let mut mint = SetOnce::<String>::new("mint");
    let mut token_program = SetOnce::<String>::new("token_program");

    let mut positional_index = 0usize;
    pl.each(|ref meta| match meta.path_str().as_str() {
        "owner" => owner.set(meta.as_value()?.as_expr()?.as_string()?, meta),
        "mint" => mint.set(meta.as_value()?.as_expr()?.as_string()?, meta),
        "token_program" => token_program.set(meta.as_value()?.as_expr()?.as_string()?, meta),
        _ => {
            let value = meta.as_expr()?.as_string()?;
            match positional_index {
                0 => {
                    positional_index += 1;
                    owner.set(value, meta)
                }
                1 => {
                    positional_index += 1;
                    mint.set(value, meta)
                }
                2 => {
                    positional_index += 1;
                    token_program.set(value, meta)
                }
                _ => Err(meta.error("too many positional arguments")),
            }
        }
    })?;

    let owner_name = owner.take(meta)?;
    let mint_name = mint.take(meta)?;
    let token_program_name = token_program.take(meta)?;

    let pda_node = PdaNode {
        name: "associatedTokenAccount".into(),
        docs: Default::default(),
        program_id: Some(ATA_PROGRAM_ID.to_string()),
        seeds: vec![
            VariablePdaSeedNode::new("owner", PublicKeyTypeNode::new()).into(),
            VariablePdaSeedNode::new("tokenProgram", PublicKeyTypeNode::new()).into(),
            VariablePdaSeedNode::new("mint", PublicKeyTypeNode::new()).into(),
        ],
    };

    Ok(PdaValueNode::new(
        pda_node,
        vec![
            PdaSeedValueNode::new("owner", AccountValueNode::new(owner_name)),
            PdaSeedValueNode::new("tokenProgram", AccountValueNode::new(token_program_name)),
            PdaSeedValueNode::new("mint", AccountValueNode::new(mint_name)),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_value, assert_value_err};

    #[test]
    fn positional() {
        assert_value!(
            { ata("escrow", "mint", "tokenProgram") },
            PdaValueNode::new(
                PdaNode {
                    name: "associatedTokenAccount".into(),
                    docs: Default::default(),
                    program_id: Some(ATA_PROGRAM_ID.to_string()),
                    seeds: vec![
                        VariablePdaSeedNode::new("owner", PublicKeyTypeNode::new()).into(),
                        VariablePdaSeedNode::new("tokenProgram", PublicKeyTypeNode::new()).into(),
                        VariablePdaSeedNode::new("mint", PublicKeyTypeNode::new()).into(),
                    ],
                },
                vec![
                    PdaSeedValueNode::new("owner", AccountValueNode::new("escrow")),
                    PdaSeedValueNode::new("tokenProgram", AccountValueNode::new("tokenProgram")),
                    PdaSeedValueNode::new("mint", AccountValueNode::new("mint")),
                ],
            )
            .into()
        );
    }

    #[test]
    fn named() {
        assert_value!(
            {
                ata(
                    owner = "escrow",
                    mint = "mint",
                    token_program = "tokenProgram",
                )
            },
            PdaValueNode::new(
                PdaNode {
                    name: "associatedTokenAccount".into(),
                    docs: Default::default(),
                    program_id: Some(ATA_PROGRAM_ID.to_string()),
                    seeds: vec![
                        VariablePdaSeedNode::new("owner", PublicKeyTypeNode::new()).into(),
                        VariablePdaSeedNode::new("tokenProgram", PublicKeyTypeNode::new()).into(),
                        VariablePdaSeedNode::new("mint", PublicKeyTypeNode::new()).into(),
                    ],
                },
                vec![
                    PdaSeedValueNode::new("owner", AccountValueNode::new("escrow")),
                    PdaSeedValueNode::new("tokenProgram", AccountValueNode::new("tokenProgram")),
                    PdaSeedValueNode::new("mint", AccountValueNode::new("mint")),
                ],
            )
            .into()
        );
    }

    #[test]
    fn missing_owner() {
        assert_value_err!({ ata("escrow", "mint") }, "token_program is missing");
    }

    #[test]
    fn too_many_args() {
        assert_value_err!({ ata("a", "b", "c", "d") }, "too many positional arguments");
    }
}
