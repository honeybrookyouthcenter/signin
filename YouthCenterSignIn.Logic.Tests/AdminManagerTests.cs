﻿using System;
using System.Linq;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class AdminTests : TestBase
    {
        [TestMethod]
        public void Admin_GetLogsTest()
        {
            var person = GetTestPerson();

            var admin = new Admin();
            var originalLogs = admin.Logs.Count;

            person.SignInOut().Wait();
            Assert.IsTrue(person.SignedIn, "After signing in, the person should be signed in.");
            Assert.AreEqual(originalLogs + 1, admin.Logs.Count, "We signed a person in, so there should be logs.");

            person.SignInOut().Wait();
            Assert.IsFalse(admin.Logs.Last().SignedIn, "The last log should be signed out.");

            person.SignInOut().Wait();
            person.SignInOut().Wait();

            var todaysLogCount = admin.Logs.Count;
            var yesterday = DateTime.Today.AddDays(-1);
            admin.Date = new DateTimeOffset(yesterday);

            var yesterdaysLogCount = admin.Logs.Count;
            Assert.AreNotEqual(yesterdaysLogCount, todaysLogCount, "The two days should have a different amount of logs");

            var newLog = Log.New(person, date: yesterday);
            Assert.AreEqual(yesterdaysLogCount + 1, admin.Logs.Count, "We signed a person in, so there should be logs.");

            admin.Date = DateTimeOffset.Now;
            Assert.AreEqual(todaysLogCount, admin.Logs.Count, "We signed a person in, so there should be logs.");
        }
        private const string NewPin = "321321";

        [TestMethod]
        public void Admin_Pin_ChangeTest()
        {
            var admin = new Admin();

            try
            {
                Assert.ThrowsException<MessageException>(() =>
                    admin.ChangeAdminPin(NewPin, "1234", "1234"));
                Assert.ThrowsException<MessageException>(() =>
                    admin.ChangeAdminPin(Admin.DefaultPin, "1234", "2345"));
                Assert.ThrowsException<MessageException>(() =>
                    admin.ChangeAdminPin(Admin.DefaultPin, "123", "123"));
                Assert.ThrowsException<MessageException>(() =>
                    admin.ChangeAdminPin(Admin.DefaultPin, NewPin, "4567"));
                Assert.AreEqual(Admin.DefaultPin, admin.Pin,
                    "We tried to change the pin incorrectly, it should not have changed.");

                admin.ChangeAdminPin(Admin.DefaultPin, NewPin, NewPin);
                Assert.AreEqual(NewPin, admin.Pin, "We changed the pin, it should have updated.");
            }
            finally
            {
                admin.ChangeAdminPin(NewPin, Admin.DefaultPin, Admin.DefaultPin);
            }

        }
        [TestMethod]
        public void Admin_Pin_AuthenticateTest()
        {
            var admin = new Admin();

            Assert.IsFalse(admin.Authenticate("1243"));
            Assert.IsTrue(admin.Authenticate(Admin.DefaultPin));
        }
    }
}
