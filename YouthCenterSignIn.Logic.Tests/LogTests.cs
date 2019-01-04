﻿using System;
using System.Collections.Generic;
using System.Linq;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class LogTests : TestBase
    {
        [TestMethod]
        public void Log_SignInOutTest()
        {
            //Clear up any tests that might have run previously
            Log.ClearCache();

            var person = GetTestPerson();
            Assert.IsFalse(person.SignedIn, "The person should not be signed in the first time.");

            person.SignInOut().Wait();
            Assert.IsTrue(person.SignedIn, "The person should not be signed in the first time.");

            person.SignInOut().Wait();
            Assert.IsFalse(person.SignedIn, "The person should not be signed in the first time.");
        }

        [TestMethod]
        public void Log_SortingTest()
        {
            //Clear up any tests that might have run previously
            Log.ClearCache();

            var people = DataProvider.GetPeople().Result;

            people[0].SignInOut().Wait();
            Assert.IsTrue(people[0].SignedIn, "The person should be signed in");

            people[1].SignInOut().Wait();
            Assert.IsTrue(people[1].SignedIn, "The person should be signed in");

            var logs = Log.GetLogs(DateTime.Now).Result.ToList();
            Assert.AreEqual(people[0].Id, logs.ElementAt(0).PersonId, "The first log should be the first person");
            Assert.AreEqual(people[1].Id, logs.ElementAt(1).PersonId, "The second log should be the second person");

            people[0].SignInOut().Wait();

            logs = Log.GetLogs(DateTime.Now).Result.ToList();
            Assert.AreEqual(people[1].Id, logs.ElementAt(0).PersonId, "First log should be the person still signed in");
            Assert.AreEqual(people[0].Id, logs.ElementAt(1).PersonId, "Second log should be the first person signed out");

            people[1].SignInOut().Wait();
        }
    }
}
