import { contractAccount, accessKeyMethods } from '_config';
const contractName = require('../../contract_name.json').contractName;

const deleteAccessKeys = async (_, res) => {
    try {
        const getAccKeys = await contractAccount.getAccessKeys();
        const accessKeys = getAccKeys.filter(({ access_key: { permission } }) => {
            return permission?.FunctionCall?.receiver_id === contractName;
        });
        const result = await Promise.all(
            accessKeys.map(({ public_key }) => await contractAccount.deleteAccount(public_key)),
        );
        res.status(200).json(result);
    } catch (err) {
        console.log('deleteAccessKeys err: ', err);
        res.status(403).json(err);
    }
};

export default deleteAccessKeys;
