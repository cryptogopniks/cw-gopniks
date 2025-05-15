use crate::cosmwasm_std;
use crate::cw_storage_plus;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, CustomQuery, DepsMut, Env, StdError, StdResult};
use cw_storage_plus::Item;

use thiserror::Error;

use crate::utils::convert_err;

/// Stores the state of changing simple process
const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer_admin_state");

#[cw_serde]
pub struct TransferAdminState {
    new_admin: Addr,
    deadline: u64,
}

impl TransferAdminState {
    pub fn get_new_admin(&self) -> Addr {
        self.new_admin.to_owned()
    }

    pub fn update_admin<Q: CustomQuery>(
        deps: &mut DepsMut<Q>,
        env: &Env,
        sender: &Addr,
        admin: &Addr,
        new_admin: &str,
        timeout: u64,
    ) -> StdResult<()> {
        Auth::simple(admin).assert(sender)?;

        let block_time = env.block.time.seconds();
        let new_admin = deps.api.addr_validate(new_admin)?;
        TRANSFER_ADMIN_STATE.save(
            deps.storage,
            &TransferAdminState {
                new_admin,
                deadline: block_time + timeout,
            },
        )
    }

    pub fn accept_admin<Q: CustomQuery>(
        deps: &mut DepsMut<Q>,
        env: &Env,
        sender: &Addr,
    ) -> StdResult<Addr> {
        let block_time = env.block.time.seconds();
        let TransferAdminState {
            new_admin,
            deadline,
        } = TRANSFER_ADMIN_STATE
            .load(deps.storage)
            .map_err(|_| AuthError::NoNewAdmin)?;

        Auth::simple(&new_admin).assert(sender)?;

        if block_time >= deadline {
            Err(AuthError::TransferAdminDeadline)?;
        }

        TRANSFER_ADMIN_STATE.update(deps.storage, |mut x| -> StdResult<_> {
            x.deadline = block_time;
            Ok(x)
        })?;

        Ok(sender.to_owned())
    }
}

#[cw_serde]
pub enum Auth {
    Simple(Addr),
    Optional(Option<Addr>),
    Specified(Vec<Addr>),
    SimpleOptional {
        simple: Addr,
        optional: Option<Addr>,
    },
    SimpleSpecified {
        simple: Addr,
        list: Vec<Addr>,
    },
    OptionalSpecified {
        optional: Option<Addr>,
        list: Vec<Addr>,
    },
    SimpleOptionalSpecified {
        simple: Addr,
        optional: Option<Addr>,
        list: Vec<Addr>,
    },
    Excluded(Vec<Addr>),
}

impl Auth {
    pub fn simple(addr: &Addr) -> Self {
        Self::Simple(addr.to_owned())
    }

    pub fn optional(api: &dyn Api, addr: &Option<impl ToString>) -> StdResult<Self> {
        Ok(Self::Optional(
            addr.as_ref()
                .map(|x| api.addr_validate(&x.to_string()))
                .transpose()?,
        ))
    }

    pub fn specified(api: &dyn Api, list: &[impl ToString]) -> StdResult<Self> {
        Ok(Self::Specified(
            list.iter()
                .map(|x| api.addr_validate(&x.to_string()))
                .collect::<StdResult<_>>()?,
        ))
    }

    pub fn simple_optional(
        api: &dyn Api,
        simple: &Addr,
        optional: &Option<impl ToString>,
    ) -> StdResult<Self> {
        Ok(Self::SimpleOptional {
            simple: simple.to_owned(),
            optional: optional
                .as_ref()
                .map(|x| api.addr_validate(&x.to_string()))
                .transpose()?,
        })
    }

    pub fn simple_specified(
        api: &dyn Api,
        simple: &Addr,
        list: &[impl ToString],
    ) -> StdResult<Self> {
        Ok(Self::SimpleSpecified {
            simple: simple.to_owned(),
            list: list
                .iter()
                .map(|x| api.addr_validate(&x.to_string()))
                .collect::<StdResult<_>>()?,
        })
    }

    pub fn optional_specified(
        api: &dyn Api,
        optional: &Option<impl ToString>,
        list: &[impl ToString],
    ) -> StdResult<Self> {
        Ok(Self::OptionalSpecified {
            optional: optional
                .as_ref()
                .map(|x| api.addr_validate(&x.to_string()))
                .transpose()?,
            list: list
                .iter()
                .map(|x| api.addr_validate(&x.to_string()))
                .collect::<StdResult<_>>()?,
        })
    }

    pub fn simple_optional_specified(
        api: &dyn Api,
        simple: &Addr,
        optional: &Option<impl ToString>,
        list: &[impl ToString],
    ) -> StdResult<Self> {
        Ok(Self::SimpleOptionalSpecified {
            simple: simple.to_owned(),
            optional: optional
                .as_ref()
                .map(|x| api.addr_validate(&x.to_string()))
                .transpose()?,
            list: list
                .iter()
                .map(|x| api.addr_validate(&x.to_string()))
                .collect::<StdResult<_>>()?,
        })
    }

    pub fn excluded(api: &dyn Api, list: &[impl ToString]) -> StdResult<Self> {
        Ok(Self::Excluded(
            list.iter()
                .map(|x| api.addr_validate(&x.to_string()))
                .collect::<StdResult<_>>()?,
        ))
    }

    pub fn assert(&self, sender: &Addr) -> StdResult<()> {
        match self {
            Auth::Simple(simple) => {
                if sender != simple {
                    Err(AuthError::Unauthorized)?;
                }
            }

            Auth::Optional(optional) => {
                if !optional.as_ref().map(|x| x == sender).unwrap_or_default() {
                    Err(AuthError::Unauthorized)?;
                }
            }

            Auth::Specified(list) => {
                if !list.contains(sender) {
                    Err(AuthError::Unauthorized)?;
                }
            }

            Auth::SimpleOptional { simple, optional } => {
                if (sender != simple) && !optional.as_ref().map(|x| x == sender).unwrap_or_default()
                {
                    Err(AuthError::Unauthorized)?;
                }
            }

            Auth::SimpleSpecified { simple, list } => {
                if (sender != simple) && !list.contains(sender) {
                    Err(AuthError::Unauthorized)?;
                }
            }

            Auth::OptionalSpecified { optional, list } => {
                if !optional.as_ref().map(|x| x == sender).unwrap_or_default()
                    && !list.contains(sender)
                {
                    Err(AuthError::Unauthorized)?;
                }
            }

            Auth::SimpleOptionalSpecified {
                simple,
                optional,
                list,
            } => {
                if (sender != simple)
                    && !optional.as_ref().map(|x| x == sender).unwrap_or_default()
                    && !list.contains(sender)
                {
                    Err(AuthError::Unauthorized)?;
                }
            }

            Auth::Excluded(list) => {
                if list.contains(sender) {
                    Err(AuthError::Unauthorized)?;
                }
            }
        };

        Ok(())
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AuthError {
    #[error("Sender doesn't have access permissions!")]
    Unauthorized,

    #[error("New admin wasn't specified!")]
    NoNewAdmin,

    #[error("It's too late to accept admin role!")]
    TransferAdminDeadline,
}

impl From<AuthError> for StdError {
    fn from(error: AuthError) -> Self {
        convert_err(error)
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Addr, Auth, AuthError, StdResult, TransferAdminState, TRANSFER_ADMIN_STATE};
    use crate::cosmwasm_std::testing::{mock_dependencies, mock_env};

    const ADMIN: &str = "cosmwasm105yqjjdgl00nzwyj9aua98zgetdn4qyhukjf5t";
    const WORKER: &str = "cosmwasm10datnnlcjmrdl37ka0g4u83chvxpfafm9t6nyr";
    const ALICE: &str = "cosmwasm10fwlvt749384x2278gylk50kxvr2lgc5qrpuud";
    const BOB: &str = "cosmwasm10n5dcumwvsaf6ma3gup0msp5w9hvu70vwl0ve7";
    const SENDER: &str = "cosmwasm10s944fgumfqtw384864xq0zl69z9gg2dqjgdes";

    struct TestAddr {
        pub admin: Addr,
        pub worker: Option<Addr>,
        pub alice: String,
        pub bob: String,
        pub sender: Addr,
    }

    fn get_addr() -> TestAddr {
        TestAddr {
            admin: Addr::unchecked(ADMIN),
            worker: Some(Addr::unchecked(WORKER)),
            alice: String::from(ALICE),
            bob: String::from(BOB),
            sender: Addr::unchecked(SENDER),
        }
    }

    #[test]
    fn test_transfer_admin() -> StdResult<()> {
        const TRANSFER_ADMIN_TIMEOUT: u64 = 100;

        let x = get_addr();
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        TRANSFER_ADMIN_STATE.load(&deps.storage).unwrap_err();
        assert_eq!(
            TransferAdminState::accept_admin(&mut deps.as_mut(), &env, &x.sender).unwrap_err(),
            AuthError::NoNewAdmin.into()
        );

        TransferAdminState::update_admin(
            &mut deps.as_mut(),
            &env,
            &x.admin,
            &x.admin,
            &x.alice,
            TRANSFER_ADMIN_TIMEOUT,
        )?;
        assert_eq!(
            TRANSFER_ADMIN_STATE.load(&deps.storage)?,
            TransferAdminState {
                new_admin: Addr::unchecked(&x.alice),
                deadline: env.block.time.seconds() + TRANSFER_ADMIN_TIMEOUT
            }
        );

        assert_eq!(
            TransferAdminState::accept_admin(&mut deps.as_mut(), &env, &x.sender).unwrap_err(),
            AuthError::Unauthorized.into()
        );

        env.block.time = env.block.time.plus_seconds(2 * TRANSFER_ADMIN_TIMEOUT);
        assert_eq!(
            TransferAdminState::accept_admin(&mut deps.as_mut(), &env, &Addr::unchecked(&x.alice))
                .unwrap_err(),
            AuthError::TransferAdminDeadline.into()
        );

        TransferAdminState::update_admin(
            &mut deps.as_mut(),
            &env,
            &x.admin,
            &x.admin,
            &x.alice,
            TRANSFER_ADMIN_TIMEOUT,
        )?;
        assert_eq!(
            TransferAdminState::accept_admin(&mut deps.as_mut(), &env, &Addr::unchecked(&x.alice))?
                .to_string(),
            x.alice
        );

        Ok(())
    }

    #[test]
    fn test_simple() -> StdResult<()> {
        let x = get_addr();

        Auth::simple(&x.admin).assert(&x.admin)?;
        assert_eq!(
            Auth::simple(&x.admin).assert(&x.sender).unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }

    #[test]
    fn test_optional() -> StdResult<()> {
        let x = get_addr();
        let deps = mock_dependencies();

        Auth::optional(&deps.api, &x.worker)?.assert(&x.worker.clone().unwrap())?;
        assert_eq!(
            Auth::optional(&deps.api, &x.worker)?
                .assert(&x.sender)
                .unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }

    #[test]
    fn test_specified() -> StdResult<()> {
        let x = get_addr();
        let deps = mock_dependencies();

        Auth::specified(&deps.api, &[&x.alice, &x.sender.to_string()])?.assert(&x.sender)?;
        assert_eq!(
            Auth::specified(&deps.api, &[&x.alice, &x.bob])?
                .assert(&x.sender)
                .unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }

    #[test]
    fn test_simple_optional() -> StdResult<()> {
        let x = get_addr();
        let deps = mock_dependencies();

        Auth::simple_optional(&deps.api, &x.admin, &x.worker)?.assert(&x.admin)?;
        Auth::simple_optional(&deps.api, &x.admin, &x.worker)?
            .assert(&x.worker.clone().unwrap())?;
        assert_eq!(
            Auth::simple_optional(&deps.api, &x.admin, &x.worker)?
                .assert(&x.sender)
                .unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }

    #[test]
    fn test_simple_specified() -> StdResult<()> {
        let x = get_addr();
        let deps = mock_dependencies();

        Auth::simple_specified(&deps.api, &x.admin, &[&x.alice, &x.sender.to_string()])?
            .assert(&x.admin)?;
        Auth::simple_specified(&deps.api, &x.admin, &[&x.alice, &x.sender.to_string()])?
            .assert(&x.sender)?;
        assert_eq!(
            Auth::simple_specified(&deps.api, &x.admin, &[&x.alice, &x.bob])?
                .assert(&x.sender)
                .unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }

    #[test]
    fn test_optional_specified() -> StdResult<()> {
        let x = get_addr();
        let deps = mock_dependencies();

        Auth::optional_specified(&deps.api, &x.worker, &[&x.alice, &x.sender.to_string()])?
            .assert(&x.worker.clone().unwrap())?;
        Auth::optional_specified(&deps.api, &x.worker, &[&x.alice, &x.sender.to_string()])?
            .assert(&x.sender)?;
        assert_eq!(
            Auth::optional_specified(&deps.api, &x.worker, &[&x.alice, &x.bob])?
                .assert(&x.sender)
                .unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }

    #[test]
    fn test_simple_optional_specified() -> StdResult<()> {
        let x = get_addr();
        let deps = mock_dependencies();

        Auth::simple_optional_specified(
            &deps.api,
            &x.admin,
            &x.worker,
            &[&x.alice, &x.sender.to_string()],
        )?
        .assert(&x.admin)?;
        Auth::simple_optional_specified(
            &deps.api,
            &x.admin,
            &x.worker,
            &[&x.alice, &x.sender.to_string()],
        )?
        .assert(&x.worker.clone().unwrap())?;
        Auth::simple_optional_specified(
            &deps.api,
            &x.admin,
            &x.worker,
            &[&x.alice, &x.sender.to_string()],
        )?
        .assert(&x.sender)?;
        assert_eq!(
            Auth::simple_optional_specified(&deps.api, &x.admin, &x.worker, &[&x.alice, &x.bob],)?
                .assert(&x.sender)
                .unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }

    #[test]
    fn test_excluded() -> StdResult<()> {
        let x = get_addr();
        let deps = mock_dependencies();

        Auth::excluded(&deps.api, &[&x.alice, &x.bob])?.assert(&x.sender)?;
        assert_eq!(
            Auth::excluded(&deps.api, &[&x.alice, &x.sender.to_string()])?
                .assert(&x.sender)
                .unwrap_err(),
            AuthError::Unauthorized.into()
        );

        Ok(())
    }
}
