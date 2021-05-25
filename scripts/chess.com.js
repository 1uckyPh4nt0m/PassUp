module.exports = {
'chess.com' : function(browser) {
	const url = process.argv[6];
	const userName = process.argv[7];
    const oldPasswd = process.argv[8];
    const newPasswd = process.argv[9];

	browser
		.url(url)
      	.waitForElementPresent('body')
		.waitForElementPresent('a.button.auth.login')
		.click('a.button.auth.login')
		.waitForElementPresent('input[name=_username]')
      	.setValue('input[name=_username]', userName)
      	.setValue('input[name=_password]', oldPasswd)
		.waitForElementPresent('button#login')
		.click('button#login')
		.waitForElementPresent('partial link text', 'Settings')
		.click('partial link text', 'Settings')
		.waitForElementPresent('partial link text', 'Password')
		.click('partial link text', 'Password')
		.waitForElementPresent('#password_currentPassword')
		.setValue('#password_currentPassword', oldPasswd)
		.setValue('#password_password_first', newPasswd)
		.setValue('#password_password_second', newPasswd)
		.click('#password_save')
		.end();
  }
};
