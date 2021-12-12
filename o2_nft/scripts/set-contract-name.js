const fs = require("fs");
const contractName = fs.readFileSync("./neardev/dev-account").toString();

const contract_name = {
    contractName,
};

fs.writeFileSync(
    "./fe/contract_name.json",
    JSON.stringify(contract_name, undefined, 4)
);
