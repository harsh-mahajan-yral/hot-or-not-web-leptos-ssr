pub mod tokens;
pub mod transactions;
mod txn;
use leptos::*;
use tokens::{TokenRootList, TokenView};

use crate::{
    component::{
        back_btn::BackButton,
        bullet_loader::BulletLoader,
        canisters_prov::{with_cans, AuthCansProvider, WithAuthCans},
        connect::ConnectLogin,
        infinite_scroller::{CursoredDataProvider, KeyedData},
    },
    state::{
        auth::account_connected_reader,
        canisters::{authenticated_canisters, Canisters},
    },
    try_or_redirect_opt,
    utils::profile::ProfileDetails,
};
use txn::{provider::get_history_provider, TxnView};

#[component]
fn ProfileGreeter(details: ProfileDetails) -> impl IntoView {
    // let (is_connected, _) = account_connected_reader();

    view! {
        <div class="flex flex-col">
            <span class="text-white/50 text-md">Welcome!</span>
            <span class="text-white text-lg md:text-xl truncate">
                // TEMP: Workaround for hydration bug until leptos 0.7
                // class=("md:w-5/12", move || !is_connected())
                {details.display_name_or_fallback()}
            </span>
        </div>
        <div class="w-16 aspect-square overflow-clip justify-self-end rounded-full">
            <img class="h-full w-full object-cover" src=details.profile_pic_or_random()/>
        </div>
    }
}

#[component]
fn FallbackGreeter() -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <span class="text-white/50 text-md">Welcome!</span>
            <div class="w-3/4 rounded-full py-2 bg-white/40 animate-pulse"></div>
        </div>
        <div class="w-16 aspect-square overflow-clip rounded-full justify-self-end bg-white/40 animate-pulse"></div>
    }
}

const RECENT_TXN_CNT: usize = 10;

#[component]
fn BalanceFallback() -> impl IntoView {
    view! { <div class="w-1/4 rounded-full py-3 mt-1 bg-white/30 animate-pulse"></div> }
}

#[component]
pub fn Wallet() -> impl IntoView {
    let (is_connected, _) = account_connected_reader();

    let auth_cans = authenticated_canisters();
    let balance_fetch = auth_cans.derive(
        || (),
        |cans_wire, _| async move {
            let cans = cans_wire?.canisters()?;
            let user = cans.authenticated_user().await;

            let bal = user.get_utility_token_balance().await?;
            Ok::<_, ServerFnError>(bal.to_string())
        },
    );
    let history_fetch = auth_cans.derive(
        || (),
        |cans_wire, _| async move {
            let cans = cans_wire?.canisters()?;
            let history_prov = get_history_provider(cans);
            let page = history_prov.get_by_cursor(0, RECENT_TXN_CNT).await?;

            Ok::<_, ServerFnError>(page.data)
        },
    );
    // let tokens_fetch = auth_cans.derive(
    //     || (),
    //     |cans_wire, _| async move {
    //         let cans = cans_wire?.canisters()?;
    //         let tokens_prov = TokenRootList(cans);
    //         let tokens = tokens_prov.get_by_cursor(0, 5).await;
    //         Ok::<_, ServerFnError>(tokens.map(|t| t.data).unwrap_or_default())
    //     },
    // );
    let tokens_fetch = with_cans(|cans: Canisters<true>| {
        let tokens_prov = TokenRootList(cans);
        async move {
            let tokens = tokens_prov.get_by_cursor(0, 5).await;
            tokens.map(|t| t.data).unwrap_or_default()
        }
    });

    view! {
        <div>
            <div class="top-0 bg-black text-white w-full items-center z-50 pt-4 pl-4">
                <div class="flex flex-row justify-start">
                    <BackButton fallback="/".to_string()/>
                </div>
            </div>
            <div class="flex flex-col w-dvw min-h-dvh bg-black gap-4 px-4 pt-4 pb-12">
                <div class="grid grid-cols-2 grid-rows-1 items-center w-full">
                    <AuthCansProvider fallback=FallbackGreeter let:cans>
                        <ProfileGreeter details=cans.profile_details()/>
                    </AuthCansProvider>
                </div>
                <div class="flex flex-col w-full items-center mt-6 text-white">
                    <span class="text-md lg:text-lg uppercase">Your Coyns Balance</span>
                    <Suspense fallback=BalanceFallback>
                        {move || {
                            let balance = try_or_redirect_opt!(balance_fetch() ?);
                            Some(view! { <div class="text-xl lg:text-2xl">{balance}</div> })
                        }}

                    </Suspense>
                </div>
                <Show when=move || !is_connected()>
                    <div class="flex flex-col w-full py-5 items-center">
                        <div class="flex flex-row w-9/12 md:w-5/12 items-center">
                            <ConnectLogin
                                login_text="Login to claim your COYNs"
                                cta_location="wallet"
                            />
                        </div>
                    </div>
                </Show>
                <div class="flex flex-col w-full gap-2">
                    <div class="flex flex-row w-full items-end justify-between">
                        <span class="text-white text-sm md:text-md">My Tokens</span>
                        <a href="/tokens" class="text-white/50 text-md md:text-lg">
                            See All
                        </a>
                    </div>
                    <div class="flex flex-col gap-2 items-center">
                        <WithAuthCans fallback=BulletLoader with=tokens_fetch let:tokens>
                            <For each=move || tokens.1.clone() key=|token| *token let:token>
                                <TokenView
                                    user_principal=tokens.0.user_principal()
                                    token_root=token
                                />
                            </For>
                        </WithAuthCans>
                    </div>
                </div>
                <div class="flex flex-col w-full gap-2">
                    <div class="flex flex-row w-full items-end justify-between">
                        <span class="text-white text-sm md:text-md">Recent Transactions</span>
                        <a href="/transactions" class="text-white/50 text-md md:text-lg">
                            See All
                        </a>
                    </div>
                    <div class="flex flex-col divide-y divide-white/10">
                        <Suspense fallback=BulletLoader>
                            {move || {
                                history_fetch()
                                    .map(|history| {
                                        view! {
                                            <For
                                                each=move || history.clone().unwrap_or_default()
                                                key=|inf| inf.key()
                                                let:info
                                            >
                                                <TxnView info/>
                                            </For>
                                        }
                                    })
                            }}

                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}
