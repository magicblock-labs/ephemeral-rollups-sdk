const path = require('path');

module.exports = {
  idlPath: path.join(__dirname, 'idls/delegation.json'),
  programId: 'DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh',
  programName: 'delegation',
  accountsDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  instructionsDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  typesDir: path.join(__dirname, 'src/generated/delegation-program-instructions'),
  removeExistingIdl: false,
  idlGenerator: 'anchor',
};
