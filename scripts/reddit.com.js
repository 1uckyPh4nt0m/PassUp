module.exports = {
    'reddit.com' : function(browser) {
        const url = 'https://www.reddit.com/login';
        const userName = process.argv[6];
        const oldPasswd = process.argv[7];
        const newPasswd = process.argv[8];

        browser
            .url(url)
            .waitForElementPresent('body')
            .waitForElementVisible('button.AnimatedForm__submitButton.m-full-width')
            .setValue('input[name=username]', userName)
            .setValue('input[name=password]', oldPasswd)
            .click('button.AnimatedForm__submitButton.m-full-width')
            .pause(5000)
            .url('https://www.reddit.com/change_password/?experiment_d2x_2020ify_buttons=enabled')
            .waitForElementPresent('input[name=old_password]')
            .setValue('input[name=old_password]', oldPasswd)
            .setValue('input[name=password]', newPasswd)
            .setValue('input[name=password2]', newPasswd)
            .click('button[type=submit]')
            .waitForElementPresent('span.AnimatedForm__submitStatusMessage')
            .assert.containsText('span.AnimatedForm__submitStatusMessage', 'has been changed!')
            .end();
    }
};