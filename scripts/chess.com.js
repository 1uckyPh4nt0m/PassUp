module.exports = {
'chess.com' : function(browser) {
	const url = process.argv[6];
	const userName = process.argv[7];
    const oldPasswd = process.argv[8];
    const newPasswd = process.argv[9];

	console.log(newPasswd);

	browser
		.url(url)
      	.waitForElementVisible('body')
		.waitForElementVisible('a.button.auth.login')
		.click('a.button.auth.login')
		.waitForElementVisible('input[name=_username]')
      	.setValue('input[name=_username]', userName)
      	.setValue('input[name=_password]', oldPasswd)
		.assert.visible('button#login')
		.click('button#login')
		.click('partial link text', 'Skip Trial')
		.waitForElementVisible('a.action.link.has-popover.settings')
		.click('a.action.link.has-popover.settings')
		.waitForElementVisible('partial link text', 'Password')
		.click('partial link text', 'Password')
		.waitForElementVisible('#password_currentPassword')
		.setValue('#password_currentPassword', oldPasswd)
		.setValue('#password_password_first', newPasswd)
		.setValue('#password_password_second', newPasswd)
		.click('#password_save')
		.assert.visible('button#login')
		.end();
  }
};
