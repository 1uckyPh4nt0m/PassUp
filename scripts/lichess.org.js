// usage:
// nightwatch --env firefox --test url userName oldPasswd newPasswd  
// nightwatch --env firefox --test http://www.lichess.org bachelor1 password123 password456
module.exports = {
'lichess.org' : function(browser) {
	const url = "https://www.lichess.org";
	const userName = process.argv[6];
    const oldPasswd = process.argv[7];
    const newPasswd = process.argv[8];

	browser
		.url(url)
      	.waitForElementPresent('body')
		.waitForElementPresent('a.signin')
		.click('a.signin')
		.waitForElementPresent('input[name=username]')
      	.setValue('input[name=username]', userName)
      	.setValue('input[name=password]', oldPasswd)
		.click('button.submit.button')
		.waitForElementPresent('#user_tag')
		.click('#user_tag')
		.waitForElementPresent('partial link text', 'Preferences')
		.click('partial link text', 'Preferences')
		.waitForElementPresent('partial link text', 'Change password')
		.click('partial link text', 'Change password')
		.waitForElementPresent('#form3-oldPasswd')
		.setValue('#form3-oldPasswd', oldPasswd)
		.setValue('#form3-newPasswd1', newPasswd)
		.setValue('#form3-newPasswd2', newPasswd)
		.click('button.submit.button.text')
		.assert.containsText('div.flash__content', 'Success')
		.end();
  }
};
