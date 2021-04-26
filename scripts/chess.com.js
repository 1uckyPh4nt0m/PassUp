module.exports = {
'chess.com' : function(browser) {
	const url = process.argv[6];
	const userName = process.argv[7];
    const oldPasswd = process.argv[8];
    const newPasswd = process.argv[9];

	//console.log(userName);

	browser
		.url(url)
      	.waitForElementVisible('body')
		.waitForElementVisible('a.button.auth.login')
		.click('a.button.auth.login')
		.waitForElementVisible('input[name=_username]')
      	.setValue('input[name=_username]', userName)
      	.setValue('input[name=_password]', oldPasswd)
		.assert.visible('button.login')
		.click('button.login')
		.waitForElementVisible('a.action.link.has-popover.settings')
		.click('a.action.link.has-popover.settings')
		.waitForElementVisible('input[name=password[currentPassword]]')
		.setValue('input[name=password[currentPassword]]', oldPasswd)
		.setValue('input[name=password[password][first]]', newPasswd)
		.setValue('input[name=password[password][second]]', newPasswd)
		.click('#password_save')
		.assert.containsText('button.login')
		.end();
  }
};
