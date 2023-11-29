const { runInteractive } = require('./vcxclient-interactive')
const { runScript } = require('./script-common')

const optionDefinitions = [
  {
    name: 'help',
    alias: 'h',
    type: Boolean,
    description: 'Display this usage guide.'
  },
  {
    name: 'acceptTaa',
    type: Boolean,
    description: 'If specified accpets taa',
    defaultValue: false
  },
  {
    name: 'seed',
    type: String,
    description: 'Provision seed',
    defaultValue: '000000000000000000000000Trustee1'
  },
  {
    name: 'name',
    type: String,
    description: 'Agent name'
  },
  {
    name: 'rustLog',
    type: String,
    description: 'Rust log level',
    defaultValue: 'warn,aries-vcx=trace'
  },
  {
    name: 'agencyUrl',
    type: String,
    description: 'Url of mediator agency',
    defaultValue: 'http://localhost:8080'
  },
  {
    name: 'indyNetwork',
    type: String,
    description: 'Identifier for indy network',
    defaultValue: '127.0.0.1'
  }
]

const usage = [
  {
    header: 'Options',
    optionList: optionDefinitions
  },
  {
    content: 'Project home: {underline https://github.com/AbsaOSS/libvcx}'
  }
]

function areOptionsValid (_options) {
  return true
}

runScript(optionDefinitions, usage, areOptionsValid, runInteractive)