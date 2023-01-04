using System;
using System.Threading.Tasks;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using SignIn.Logic.Data;

namespace SignIn.Logic.Tests
{
    [TestClass]
    public class PersonUpdateInfoTests : TestBase
    {
        [TestMethod]
        public async Task Person_UpdateInfo_IsInfoExpiredTest()
        {
            var expiredPerson = GetTestPerson("Expired");
            Assert.IsTrue(expiredPerson.IsInfoExpired, "This person's info is out of date and should be expired.");
            Assert.AreEqual(SignInOutResult.InfoExpired, await expiredPerson.SignInOut());

            expiredPerson.SkipNextExpire = true;
            Assert.AreEqual(SignInOutResult.Success, await expiredPerson.SignInOut(), "Should sign in since skipping");
            Assert.IsFalse(expiredPerson.SkipNextExpire, "Should set back to false");
        }

        [TestMethod]
        public async Task Person_UpdateInfo_IsInfoExpired_NotAskedTest()
        {
            var notAskedPerson = GetTestPerson("Not Asked");
            Assert.IsTrue(notAskedPerson.IsInfoExpired, "This person's was never checked and should be expired.");
            Assert.AreEqual(SignInOutResult.InfoExpired, await notAskedPerson.SignInOut());
        }

        [TestMethod]
        public void Person_UpdateInfo_IsInfoExpired_UpToDateTest()
        {
            var upToDate = GetTestPerson();
            Assert.IsFalse(upToDate.IsInfoExpired, "This person's info is up to date.");
        }

        [TestMethod]
        public void Person_UpdateInfo_IsInfoExpired_NewTest()
        {
            new Person()
            {
                FirstName = "Assigned",
                LastName = "Info",
                Address = new Address("123 Programmer St", "Geekville", "UT"),
                BirthDate = DateTimeOffset.Now.AddYears(-10),
                Grade = "1",
                Guardian = new Guardian()
                {
                    Name = "Correct Info",
                    PhoneNumber = "717-334-3334"
                }
            }.Save().Wait();

            var createdPerson = GetTestPerson("Assigned Info");
            Assert.IsFalse(createdPerson.IsInfoExpired, "When creating a new person the info should be up to date.");
        }
    }
}
