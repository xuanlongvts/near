import nearAPI from 'near-api-js';

import { contractAccount, accessKeyMethods } from '_config';
const contractName = require('../../contract_name.json').contractName;

const {
    utils: {
        format: { parseNearAmount },
    },
} = nearAPI;

const addKey = async (req, res) => {
    try {
        const { publicKey } = req.body;
        const result = await contractAccount.addKey(
            publicKey,
            contractName,
            accessKeyMethods.changeMethods,
            parseNearAmount('1'),
        );
        res.status(200).json(result);
    } catch (err) {
        console.log('addKey err: ', err);
        res.status(403).json('Key is already added');
    }
};

export default addKey;
