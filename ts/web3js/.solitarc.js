const path = require('path');

module.exports = {
<<<<<<< HEAD
  idlUrl: 'https://raw.githubusercontent.com/magicblock-labs/delegation-program/main/idl/delegation.json',
  programId: 'DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh',
  idlGenerator: 'anchor',
  accountsDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  instructionsDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  typesDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
=======
  idlPath: path.join(__dirname, 'idls/delegation.json'),
  programId: 'DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh',
  programName: 'delegation',
  accountsDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  instructionsDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  typesDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  removeExistingIdl: false,
  idlGenerator: 'anchor',
>>>>>>> jonasXchen/action-escrow
};
