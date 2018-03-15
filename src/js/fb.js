/*
 *  Senie
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

/* eslint-disable prefer-const */
/* eslint-disable no-unused-vars */

export function initFirebase() {
  // Initialize Firebase
  const config = {
    apiKey: 'AIzaSyAQ5NbHbcc8QgqZLYVOB6BlS-QKQH6up-o',
    authDomain: 'project-719123155808814006.firebaseapp.com',
    databaseURL: 'https://project-719123155808814006.firebaseio.com',
    storageBucket: 'project-719123155808814006.appspot.com'
  };
  firebase.initializeApp(config);
}


/**
 * Function called when clicking the Login/Logout button.
 */
function toggleSignIn() {
  if (!firebase.auth().currentUser) {
    const provider = new firebase.auth.GoogleAuthProvider();
    provider.addScope('https://www.googleapis.com/auth/plus.login');
    firebase.auth().signInWithRedirect(provider);
  } else {
    firebase.auth().signOut();
  }
  document.getElementById('firebase-sign-in').disabled = true;
}

/**
 * initFirebaseSignIn handles setting up the Firebase context and registering
 * callbacks for the auth status.
 *
 * The core initialization is in firebase.App - this is the glue class
 * which stores configuration. We provide an app name here to allow
 * distinguishing multiple app instances.
 *
 * This method also registers a listener with
 * firebase.auth().onAuthStateChanged.
 * This listener is called when the user is signed in or out, and that
 * is where we update the UI.
 *
 * When signed in, we also authenticate to the Firebase Realtime Database.
 */
export function initFirebaseSignIn() {
  // Result from Redirect auth flow.
  firebase.auth().getRedirectResult().then(result => {
    if (result.credential) {
      // This gives you a Google Access Token.
      // You can use it to access the Google API.
      let token = result.credential.accessToken;
    }
    // The signed-in user info.
    // let user = result.user;
  }).catch(error => {
    // Handle Errors here.
    let errorCode = error.code;
    let errorMessage = error.message;
    // The email of the user's account used.
    let email = error.email;
    // The firebase.auth.AuthCredential type that was used.
    let credential = error.credential;
    if (errorCode === 'auth/account-exists-with-different-credential') {
      // alert('signed up with different auth provider for that email.');
      // If you are using multiple auth providers on your app
      // you should handle linking the user's accounts here.
    } else {
      console.error(error);
    }
  });

  const fbSignIn = document.getElementById('firebase-sign-in');

  // Listening for auth state changes.
  firebase.auth().onAuthStateChanged(user => {
    if (user) {
      // User is signed in.
      let displayName = user.displayName;
      let email = user.email;
      let emailVerified = user.emailVerified;
      let photoURL = user.photoURL;
      let isAnonymous = user.isAnonymous;
      let uid = user.uid;
      let refreshToken = user.refreshToken;
      let providerData = user.providerData;
      fbSignIn.textContent = displayName;

      const info =
        JSON.stringify({
          displayName,
          email,
          emailVerified,
          photoURL,
          isAnonymous,
          uid,
          refreshToken,
          providerData
        }, null, '  ');
    } else {
      // User is signed out.
      fbSignIn.textContent = 'Sign In';
    }
    fbSignIn.disabled = false;
  });
  fbSignIn.addEventListener('click', toggleSignIn, false);
}
