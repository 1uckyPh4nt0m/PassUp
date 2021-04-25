// usage:
// nightwatch --env firefox --test lichess.org.js url userName oldPasswd newPasswd  
// nightwatch --env firefox --test lichess.org.js http://www.lichess.org bachelor1 password123 password456
module.exports = {
'lichess.org' : function(browser) {
	const url = process.argv[6];
	const userName = process.argv[7];
    const oldPasswd = process.argv[8];
    const newPasswd = process.argv[9];

	//console.log(userName);

	browser
		.url(url)
      	.waitForElementVisible('body')
		.waitForElementVisible('a.signin')
		.click('a.signin')
		.waitForElementVisible('input[name=username]')
      	.setValue('input[name=username]', userName)
      	.setValue('input[name=password]', oldPasswd)
		.assert.visible('button.submit.button')
		.click('button.submit.button')
		.waitForElementVisible('#user_tag')
		.assert.visible('#user_tag')
		.click('#user_tag')
		.waitForElementVisible('partial link text', 'Preferences')
		.click('partial link text', 'Preferences')
		.waitForElementVisible('partial link text', 'Change password')
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
