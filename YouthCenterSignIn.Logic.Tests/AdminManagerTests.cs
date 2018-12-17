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
            var originalLogs = adminManager.TodaysLogs.Count;

            person.SignInOut().Wait();
            Assert.IsTrue(person.SignedIn, "After signing in, the person should be signed in.");
            Assert.AreEqual(originalLogs + 1, adminManager.TodaysLogs.Count, "We signed a person in, so there should be logs in the manager.");

            person.SignInOut().Wait();
            Assert.IsFalse(adminManager.TodaysLogs.Last().SignedIn, "The last log should be signed out.");
        }
    }
}
