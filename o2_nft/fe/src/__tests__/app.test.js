import nearAPI from 'near-api-js';

const contractName = require('../../contract_name.json').contractName;

describe(`Deploy contract ${contractName}`, () => {
    console.log('env: ', process.env);
    test('1. Demo test', () => {
        expect(4).toEqual(4);
    });
});
