using System;
using System.Linq;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class AdminManagerTests : TestBase
    {
        [TestMethod]
        public void AdminManager_GetLogsTest()
        {
            var person = GetTestPerson();

            var adminManager = new AdminManager();
            var originalLogs = adminManager.Logs.Count;

            person.SignInOut().Wait();
            Assert.IsTrue(person.SignedIn, "After signing in, the person should be signed in.");
            Assert.AreEqual(originalLogs + 1, adminManager.Logs.Count, "We signed a person in, so there should be logs in the manager.");

            person.SignInOut().Wait();
            Assert.IsFalse(adminManager.Logs.Last().SignedIn, "The last log should be signed out.");

            person.SignInOut().Wait();
            person.SignInOut().Wait();

            var todaysLogCount = adminManager.Logs.Count;
            var yesterday = DateTime.Today.AddDays(-1);
            adminManager.Date = new DateTimeOffset(yesterday);

            var yesterdaysLogCount = adminManager.Logs.Count;
            Assert.AreNotEqual(yesterdaysLogCount, todaysLogCount, "The two days should have a different amount of logs");

            var newLog = Log.New(person, date: yesterday);
            Assert.AreEqual(yesterdaysLogCount + 1, adminManager.Logs.Count, "We signed a person in, so there should be logs in the manager.");

            adminManager.Date = DateTimeOffset.Now;
            Assert.AreEqual(todaysLogCount, adminManager.Logs.Count, "We signed a person in, so there should be logs in the manager.");
        }
    }
}
