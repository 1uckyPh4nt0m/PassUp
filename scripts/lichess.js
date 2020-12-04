// usage:
// nightwatch --env firefox lichess.js url userName oldPasswd newPasswd  
// nightwatch --env firefox --test  lichess.js http://www.lichess.org bachelor1 password123 password456
module.exports = {
'Demo test lichess.org' : function(browser) {
	const url = process.argv[7];
	const userName = process.argv[8];
    const oldPasswd = process.argv[9];
    const newPasswd = process.argv[10];

	browser
		.url(url)
	// const oldPasswd = 'password123';
    // const newPasswd = 'password456';
    // const userName = 'bachelor1';
    // browser
	// .url('https://www.lichess.org/')
      	.waitForElementVisible('body')
		.click('a.signin.button')
      	.setValue('#form3-username', userName)
      	.setValue('#form3-password', oldPasswd)
		.assert.visible('button.submit.button')
		.click('button.submit.button')
		.waitForElementVisible('#user_tag')
		.assert.visible('#user_tag')
		.click('#user_tag')
		.click('partial link text', 'Preferences')
		.click('partial link text', 'Change password')
		.waitForElementVisible('#form3-oldPasswd')
		.setValue('#form3-oldPasswd', oldPasswd)
		.setValue('#form3-newPasswd1', newPasswd)
		.setValue('#form3-newPasswd2', newPasswd)
		.click('button.submit.button.text')
		.assert.containsText('div.flash__content', 'Success')
		.end();
  }
};
