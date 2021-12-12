import nearAPI from 'near-api-js';

const contractName = require('../../contract_name.json').contractName;
const credentials = require('../../credentials.json');

const contractMethods = {
    changeMethods: ['new', 'mint_token', 'guest_mint', 'nft_transfer', 'set_price', 'purchase', 'withdraw'],
    viewMethods: ['get_token_data', 'get_num_tokens', 'get_proceeds'],
};

export const accessKeyMethods = {
    changeMethods: ['guest_mint', 'set_price', 'withdraw'],
    viewMethods: ['get_token_data', 'get_num_tokens', 'get_proceeds', 'get_pubkey_minted'],
};

const {
    keyStores: { InMemoryKeyStore },
    Near,
    Account,
    Contract,
    KeyPair,
} = nearAPI;

export const keyStore = new InMemoryKeyStore();
keyStore.setKey(networkId, contractName, KeyPair.fromString(credentials.private_key));

export const near = new Near({
    networkId,
    nodeUrl,
    deps: { keyStore },
});

export const { connection } = near;

export const contractAccount = new Account(connection, contractName);

export const contract = new Contract(contractAccount, contractName, contractMethods);
