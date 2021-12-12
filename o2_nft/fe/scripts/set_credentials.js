const fs = require('fs');
const contractName = require('../contract_name.json').contractName;

const environment = process.argv[2];

const credentials = JSON.parse(
    fs.readFileSync(`${process.env.HOME}/.near-credentials/${environment}/${contractName}.json`),
);

fs.writeFileSync('./credentials.json', JSON.stringify(credentials, undefined, 4));
