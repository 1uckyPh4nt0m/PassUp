module.exports = {
'google.com' : function(browser) {
	const url = process.argv[6];
	const userName = process.argv[7];
    const oldPasswd = process.argv[8];
    const newPasswd = process.argv[9];

	browser
		.url(url)
      	.waitForElementPresent('body')
		.waitForElementPresent('partial link text', 'Go to Google Account')
		.click('partial link text', 'Go to Google Account')
		// .waitForElementPresent('partial link text', 'Email')
		// .setValue('partial link text', 'Email', userName)
		.waitForElementPresent('input[name=identifier]')
      	.setValue('input[name=identifier]', userName)
		.click('button[type=button]')
		.waitForElementPresent('input[id=recoveryIdentifierId')
		.setValue('input[id=recoveryIdentifierId]', userName)
		.click('button[type=button]')
		.waitForElementPresent('input[name=password]')
      	.setValue('input[name=password]', oldPasswd)
		.click('button[type=button]')
		.waitForElementPresent('div.GWwaOc')
		.click('a.GWwaOc')
		.waitForElementPresent('a.VZLjze')
		.click('a.VZLjze')
		.waitForElementPresent('input[name=password]')
      	.setValue('input[name=password]', oldPasswd)
		.click('button[type=button]')
		.waitForElementPresent('input[name=password]')
      	.setValue('input[name=password]', newPasswd)
		.setValue('input[name=confirmation_password]', newPasswd)
		.click('button[type=button]')
		.waitForElementPresent('a.VZLjze')
		.end();
  }
};
