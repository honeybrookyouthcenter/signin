using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class AddressTests : TestBase
    {
        [TestMethod]
        public void Address_AllBlank_IsValidTest()
        {
            var address = new Address("", "\r\n", null);
            Assert.IsFalse(address.IsValid(), "Everything is blank, the address is not valid.");
        }

        [TestMethod]
        public void Address_OneBlank_IsValidTest()
        {
            var address = new Address("  ", "Honey Brook", "PA");
            Assert.IsFalse(address.IsValid(), "The street is blank, the address is not valid.");
        }

        [TestMethod]
        public void Address_ValidAddress_IsValidTest()
        {
            var address = new Address("52 Evergreen St", "Gordonville", "PA");
            Assert.IsTrue(address.IsValid(), "Everything is filled out, the address is valid.");
        }
    }
}
