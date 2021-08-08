module.exports = {
    'reddit.com' : function(browser) {
        const url = 'https://www.reddit.com/login/?experiment_d2x_2020ify_buttons=enabled&experiment_d2x_sso_login_link=enabled';
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
            .waitForElementPresent('button#USER_DROPDOWN_ID')
            .click('button#USER_DROPDOWN_ID')
            .waitForElementPresent('partial link text', 'User Settings')
            .click('partial link text', 'User Settings')
            .waitForElementPresent('button._2iuoyPiKHN3kfOoeIQalDT._2tU8R9NTqhvBrhoNAXWWcP.HNozj_dKjQZ59ZsfEegz8');
        var change = browser.elements('button._2iuoyPiKHN3kfOoeIQalDT._2tU8R9NTqhvBrhoNAXWWcP.HNozj_dKjQZ59ZsfEegz8');
        console.log(change);
        browser
            .click('button._2iuoyPiKHN3kfOoeIQalDT._2tU8R9NTqhvBrhoNAXWWcP.HNozj_dKjQZ59ZsfEegz8')
            .waitForElementPresent('input[name=old_password]')
            .setValue('input[name=old_password]', oldPasswd)
            .setValue('input[name=password]', newPasswd)
            .setValue('input[name=password2]', newPasswd)
            .click('button[type=submit]')
            .assert.containsText('User settings')
            .end();
    }
};