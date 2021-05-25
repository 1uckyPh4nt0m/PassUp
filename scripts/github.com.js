module.exports = {
    'github.com' : function(browser) {
        const url = process.argv[6];
        const userName = process.argv[7];
        const oldPasswd = process.argv[8];
        const newPasswd = process.argv[9];
    
        browser
            .url(url)
            .waitForElementPresent('body')
            .waitForElementPresent('partial link text', 'Sign in')
            .click('partial link text', 'Sign in')
            .waitForElementPresent('#login_field')
            .setValue('#login_field', userName)
            .setValue('#password', oldPasswd)
            .click('input[type=submit]')
            .waitForElementPresent('img.avatar-user.avatar.avatar-small')
            .click('img.avatar-user.avatar.avatar-small')
            .waitForElementPresent('partial link text', 'Settings')
            .click('partial link text', 'Settings')
            .waitForElementPresent('partial link text', 'Account security')
            .click('partial link text', 'Account security')
            .waitForElementPresent('#user_old_password')
            .setValue('#user_old_password', oldPasswd)
            .setValue('#user_new_password', newPasswd)
            .setValue('#user_confirm_new_password', newPasswd)
            .click('button.btn.mr-2')
            .waitForElementPresent('div.flash.flash-full.flash-notice')
            .end();
    }
};