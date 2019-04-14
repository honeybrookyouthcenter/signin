using System;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class PersonUpdateInfoTests : TestBase
    {
        public void Person_UpdateInfo_IsInfoExpiredTest()
        {
            var expiredPerson = GetTestPerson("Expired");
            Assert.IsTrue(expiredPerson.IsInfoExpired, "This person's info is out of date and should be expired.");
        }

        public void Person_UpdateInfo_IsInfoExpired_NotAskedTest()
        {
            var notAskedPerson = GetTestPerson("NotAsked");
            Assert.IsTrue(notAskedPerson.IsInfoExpired, "This person's was never checked and should be expired.");
        }

        public void Person_UpdateInfo_IsInfoExpired_UpToDateTest()
        {
            var upToDate = GetTestPerson();
            Assert.IsFalse(upToDate.IsInfoExpired, "This person's info is up to date.");
        }

        public void Person_UpdateInfo_IsInfoExpired_NewTest()
        {
            new Person()
            {
                FirstName = "Assigned",
                LastName = "Info",
                Address = new Address("123 Programmer St", "Geekville", "UT"),
                BirthDate = DateTimeOffset.Now.AddYears(-10),
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
