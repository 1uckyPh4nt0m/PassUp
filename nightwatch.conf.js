const Services = {}; 
loadServices();

var port = process.env.PORT;


module.exports = {
  "output_folder": "./reports",
  test_settings: {
    firefox: {
      desiredCapabilities : {
        browserName : 'firefox',
        acceptSslCerts: true,
        alwaysMatch: {
          'moz:firefoxOptions': {
            args: [
               //'-headless',
               '-verbose'
            ],
          }
        }
      },
      webdriver: {
        port: port,
        start_process: true,
        server_path: (Services.geckodriver ? Services.geckodriver.path : ''),
        cli_args: [
          // '-vv'
        ]
      }
    },

    chrome: {
      desiredCapabilities : {
        browserName : 'chrome',
        chromeOptions : {
          args: [
            //'--no-sandbox',
            //'--ignore-certificate-errors',
            //'--allow-insecure-localhost',
            //'--headless'
          ]
        }
      },

      webdriver: {
        start_process: true,
        server_path: (Services.chromedriver ? Services.chromedriver.path : ''),
        port: port,
        cli_args: [
          // --verbose
        ]
      }
    }
  }
};

function loadServices() {
  try {
    Services.chromedriver = require('chromedriver');
  } catch (err) {}

  try {
    Services.geckodriver = require('geckodriver');
  } catch (err) {}
}