using System;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class PersonTests : TestBase
    {
        [TestMethod]
        public void Person_HasValidInfoTest()
        {
            var person = new Person();

            Assert.IsFalse(person.HasValidInfo(out string issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name, last name and address.\r\nPlease select your birth date.", issues,
                "The first name, last name and address are blank and there is an invalid date of birth.");

            person.LastName = "Esh";
            Assert.IsFalse(person.HasValidInfo(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name and address.\r\nPlease select your birth date.", issues,
                "The first name and address are blank and there is an invalid date of birth.");

            person.BirthDate = DateTimeOffset.Now.AddYears(-2);
            Assert.IsFalse(person.HasValidInfo(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name and address.\r\nYou have to be at least six to sign up.", issues,
                "The first name and address are blank and there is still an invalid date of birth.");

            person.BirthDate = new DateTimeOffset(new DateTime(1999, 2, 23));
            Assert.IsFalse(person.HasValidInfo(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name and address.\r\n", issues,
                "The first name and address are blank but the birth date is valid.");

            person.Address = "52 Evergreen St, Gordonville PA 1759";
            Assert.IsFalse(person.HasValidInfo(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name.\r\n", issues,
                "The first name is blank.");

            person.FirstName = "James";
            Assert.IsTrue(person.HasValidInfo(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("", issues, "There are no issues so the issues message should be blank");
        }
    }
}
