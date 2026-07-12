## Seperation of Concerns

core code lives in rust, app code is just for ui and hooking into os.

## General

utalize ethereum-rpc first, per user config
non ethereum-rpc network requests must be gated behind vendor flags
icons must be cached upon first fetch and not make network requests after visibility

assets enabled per account are user controlled
private material must remain inside rust except

`blo` for address preview icons
ENS support anywhere addresses can be input
ENS reverse resolution anywhere addresses are shown

all signature / transaction / export-keys actions must require explicit user authentication / approval
biometric, a pin, and / or a slider.

## Networks

prefer `network` over `chain`, `network_id` over `chain_id`.
ethereum mainnet and testnets support is the priority, stay L2 neutral do not suggest or favor any specific.

## Assets

prefer `asset` over `token`. assets are stored system wide.
assets are enabled or disabled on a per-account basis.

asset discovery using third party api's may exclusively be surfaced as light suggestions in asset management not auto enabled

there should only be a single "suggested asset list per network" in the codebase, in rust, and it should be opt-in during onboarding.

last balance fetches should be retained and returned on initial query to provide fast and snappy ui.
ui must clearly show balance quote age; clear indicators when refreshing, and auto refresh when app is opened and data is stale.

### Fiat support

We support pretty much any fiat currency the ECB supports quoting for.
Any currency should be usable (our predefined suggested list should only be defined once in rust code) and configurable in settings as display currency.

European Central Bank data can be used with `eth-prices`, this should be gated behind opt-in vendor flag.

## Price Quoting

`eth-prices` is used for price fetching and quoting.
quoters are stored system wide.

latest quotes should be retained and returned on initial query (of balances, prices, etc) to provide fast and snappy ui.
ui must clearly show quote age; clear indicators when refreshing, and auto refreshing when app is opened and data is stale.

## Transactions / History

Transaction history should be stored local-first and fetched from ethereum rpc.
3rd party api usage for transactions should be used exclusively for txhash discovery, any other data about the tx should come directly from the ethereum rpc.

When indexing a transaction we should
- figure out what part we were (avoid showing `you` instead show the account name, fallback ens name, fallback address)
- keep track of events emitted (erc20/721 transfers, approve, etc)
- keep track of function signature called

there should only be a single "decodableFunctionSignature" map, maintained in rust, that should consist of generally known / accepted / used standards, such as erc20,721,etc.

We should try and fetch verified contract code where possible, including properly resolving proxy contracts and displaying proxy contract existence accordingly.

### Optional Sourcify Vendor Flag

When enabled use the `sourcify` rust crate to fetch verified contract code (in the future other sources too).
This allows for displaying raw contract source code, aswell as advanced decoding of calldata and events.

## Verification & Testing

verification of wallet functionality should happen via https://wallet.page

## Onboarding

onboarding steps:

- setup pin
  - optionally biometric
- preferred display currency
- preferred appearance (light / dark), token units, and other visual preferences
- preferred third party apis
- setup or import account flow

## QR Scanning

EIP-681, and openlv wallet-dapp connectivity.

## Other modules

### opt-in ens renewal reminder setting

This keeps track and sets background triggers to trigger re-validation of ens-expiry X amount of time before expiry is due. And then can send notifications when names expire.
Secondary option would allow it to monitor a specific user-configurable list of names, or to have it fetch that list based on what names the user holds.

## Baseline disagreement

If you disagree with baseline.md or think something might not be possible; do not modify baseline.md; consult the user with a well rounded case for why a change to the baseline is needed. The baseline must be respected at all times. It must be short, concise, and to the point, ask if clarification is needed.
